use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

use crate::types::configuration_space::capability::Capability;
use crate::types::configuration_space::header::CommonHeaderOffset;
use crate::types::configuration_space::header::HeaderCommon;
use crate::types::configuration_space::status::PciStatus;
use crate::types::function::PciTypeFunctionCommon;

pub mod capability;
pub mod interrupt;

pub(crate) mod header;
mod status;

pub struct ConfigurationSpace {
    buf: [u8; 4096],
    next_capability_pointer: u8,
    next_available_capability_pointer: u8,
}

impl ConfigurationSpace {
    pub(crate) fn new() -> Self {
        ConfigurationSpace {
            buf: [0; 4096],
            next_capability_pointer: CommonHeaderOffset::CapabilityPointer as u8,
            next_available_capability_pointer: CommonHeaderOffset::CapabilityStart as u8,
        }
    }

    pub(crate) fn init<T: PciTypeFunctionCommon>(
        &mut self,
        header_type: u8,
        capabilities: &[Capability],
    ) {
        let header = self.as_common_header_mut();
        header.vendor_id = T::VENDOR_ID;
        header.device_id = T::DEVICE_ID;
        header.prog_if = T::CLASS_CODE as u8;
        header.subclass = (T::CLASS_CODE >> 8) as u8;
        header.class_code = (T::CLASS_CODE >> 16) as u8;
        header.header_type = header_type;

        for cap in capabilities {
            self.alloc_capability(cap);
        }
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

    /// cap: The whole cap including cap_id and next
    fn alloc_capability(&mut self, cap: &Capability) {
        let cap_len: u8 = cap.0.len().try_into().unwrap();
        let header = self.as_common_header_mut();
        header.status |= PciStatus::CapList as u16;

        let offset = self.next_available_capability_pointer;

        self.buf[self.next_capability_pointer as usize] = offset;
        self.next_capability_pointer = offset + 1;
        self.next_available_capability_pointer = offset + cap_len;

        let new_cap = &mut self.buf[(offset as usize)..((offset + cap_len) as usize)];
        new_cap.copy_from_slice(&cap.0);
        new_cap[1] = 0; // next
    }
}
