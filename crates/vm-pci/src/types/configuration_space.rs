use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

use crate::device::capability::PciCapId;
use crate::device::function::PciTypeFunctionCommon;
use crate::error::Error;
use crate::types::configuration_space::header::CommonHeaderOffset;
use crate::types::configuration_space::header::HeaderCommon;
use crate::types::configuration_space::status::PciStatus;

pub(crate) mod header;
mod status;

pub struct ConfigurationSpace {
    buf: [u8; 4096],
    last_capability_next_pointer: u8,
    next_available_capability_pointer: u16, // use u16 to check availability, otherwise `offset + cap_len` will overflow
    last_ext_capability_next_pointer: Option<u16>,
    next_available_ext_capability_pointer: u16,
}

impl ConfigurationSpace {
    pub(crate) fn new() -> Self {
        ConfigurationSpace {
            buf: [0; 4096],
            last_capability_next_pointer: CommonHeaderOffset::CapabilityPointer as u8,
            next_available_capability_pointer: CommonHeaderOffset::CapabilityStart as u16,
            last_ext_capability_next_pointer: None,
            next_available_ext_capability_pointer: CommonHeaderOffset::ExtendedCapabilityStart
                as u16,
        }
    }

    pub(crate) fn init<T: PciTypeFunctionCommon>(&mut self, header_type: u8) {
        let header = self.as_common_header_mut();
        header.vendor_id = T::VENDOR_ID;
        header.device_id = T::DEVICE_ID;
        header.prog_if = T::CLASS_CODE as u8;
        header.subclass = (T::CLASS_CODE >> 8) as u8;
        header.class_code = (T::CLASS_CODE >> 16) as u8;
        header.header_type = header_type;
    }

    pub(crate) fn as_common_header_mut(&mut self) -> &mut HeaderCommon {
        self.as_header_mut::<HeaderCommon>()
    }

    pub(crate) fn as_header_mut<T>(&mut self) -> &mut T
    where
        T: IntoBytes + FromBytes + KnownLayout + Immutable,
    {
        T::mut_from_bytes(&mut self.buf[0..size_of::<T>()]).unwrap()
    }

    pub(crate) fn read(&self, offset: u16, buf: &mut [u8]) {
        buf.copy_from_slice(&self.buf[offset as usize..offset as usize + buf.len()]);
    }

    pub(crate) fn write(&mut self, offset: u16, buf: &[u8]) {
        self.buf[offset as usize..offset as usize + buf.len()].copy_from_slice(buf);
    }

    /// cap_len: The whole length including cap_id and next
    pub fn alloc_capability(&mut self, cap_id: PciCapId, data: &[u8]) -> Result<(), Error> {
        let header = self.as_common_header_mut();
        header.status |= PciStatus::CapList as u16;

        let offset = self.next_available_capability_pointer;
        let offset = offset.next_multiple_of(4);

        let cap_len = data.len() + 2;

        if (offset as usize + cap_len) > 0x100 {
            return Err(Error::CapNoSpace);
        }

        if cap_id as u16 != cap_id as u8 as u16 {
            return Err(Error::InvalidCapId);
        }

        self.buf[self.last_capability_next_pointer as usize] = offset as u8;
        self.last_capability_next_pointer = offset as u8 + 1;
        self.next_available_capability_pointer = offset + cap_len as u16;

        let new_cap = &mut self.buf[(offset as usize)..((offset + cap_len as u16) as usize)];
        new_cap.fill(0);
        new_cap[0] = cap_id as u8;
        new_cap[2..].copy_from_slice(data);

        Ok(())
    }

    /// cap_len: The whole length including cap_id and next
    pub fn alloc_ext_capability(
        &mut self,
        cap_id: PciCapId,
        cap_version: u8,
        data: &[u8],
    ) -> Result<(), Error> {
        let offset = self.next_available_ext_capability_pointer;
        let offset = offset.next_multiple_of(4);

        let cap_len = data.len() + 4;

        if (offset as usize + cap_len) > 4096 {
            return Err(Error::CapNoSpace);
        }

        if cap_version & !0xf != 0 {
            return Err(Error::InvalidCapVersion);
        }

        if let Some(ptr) = self.last_ext_capability_next_pointer {
            let orig = u16::from_le_bytes([self.buf[ptr as usize], self.buf[ptr as usize + 1]]);

            let version = orig & 0xf;
            let val = (offset << 4) | version;

            self.buf[ptr as usize..ptr as usize + 2].copy_from_slice(&val.to_le_bytes());
        }
        self.last_ext_capability_next_pointer = Some(offset + 2);
        self.next_available_ext_capability_pointer = offset + cap_len as u16;

        let new_cap = &mut self.buf[(offset as usize)..((offset + cap_len as u16) as usize)];
        new_cap.fill(0);
        let header = ((cap_version as u32) << 16) | (cap_id as u32);
        new_cap[..4].copy_from_slice(&header.to_le_bytes());
        new_cap[4..].copy_from_slice(data);

        Ok(())
    }
}
