use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

use crate::types::configuration_space::capability::PciCapId;
use crate::types::configuration_space::capability::msix::MsixCap;
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

    pub(crate) fn init<T: PciTypeFunctionCommon>(header_type: u8) -> Self {
        let mut cfg = ConfigurationSpace::new();

        let header = cfg.as_common_header_mut();
        header.vendor_id = T::VENDOR_ID;
        header.device_id = T::DEVICE_ID;
        header.prog_if = T::CLASS_CODE as u8;
        header.subclass = (T::CLASS_CODE >> 8) as u8;
        header.class_code = (T::CLASS_CODE >> 16) as u8;
        header.header_type = header_type;

        T::init_capability(&mut cfg);

        cfg
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

    pub fn alloc_msix_capability(&mut self) -> &mut MsixCap {
        let buf = self.alloc_capability(PciCapId::MsiX, size_of::<MsixCap>().try_into().unwrap());

        MsixCap::mut_from_bytes(buf).unwrap()
    }

    /// len: the whole length of the capability
    pub fn alloc_capability(&mut self, cap_id: PciCapId, cap_len: u8) -> &mut [u8] {
        let header = self.as_common_header_mut();
        header.status |= PciStatus::PciStatusCapList as u16;

        let offset = self.next_available_capability_pointer;

        self.buf[self.next_capability_pointer as usize] = offset;
        self.next_capability_pointer = offset + 1;
        self.next_available_capability_pointer = offset + cap_len;

        let cap = &mut self.buf[(offset as usize)..((offset + cap_len) as usize)];
        cap[0] = cap_id as u8;
        cap[1] = 0; // next

        cap
    }
}
