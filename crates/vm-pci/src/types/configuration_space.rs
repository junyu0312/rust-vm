use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

use crate::types::configuration_space::capability::PciCapId;
use crate::types::configuration_space::capability::msix::MsixCap;
use crate::types::configuration_space::common::CommonHeaderOffset;
use crate::types::configuration_space::common::HeaderCommon;
use crate::types::configuration_space::common::status::PciStatus;
use crate::types::function::PciTypeFunctionCommon;

pub mod capability;
pub mod common;
pub mod interrupt;
pub mod type0;
pub mod type1;

const FIRST_CAPABILITY_OFFSET: u8 = 0x40;

pub struct ConfigurationSpace {
    buf: [u8; 4096],
    next_capability_pointer: u8,
    next_available_capability_pointer: u8,
}

impl ConfigurationSpace {
    fn new() -> Self {
        let mut buf = [0; 4096];
        buf[CommonHeaderOffset::InterruptLine as usize] = 0xff;

        ConfigurationSpace {
            buf,
            next_capability_pointer: CommonHeaderOffset::CapabilityPointer as u8,
            next_available_capability_pointer: FIRST_CAPABILITY_OFFSET,
        }
    }

    pub fn init<T: PciTypeFunctionCommon>(header_type: u8) -> Self {
        let mut cfg = ConfigurationSpace::new();

        let header = cfg.as_common_header_mut();
        header.vendor_id = T::VENDOR_ID;
        header.device_id = T::DEVICE_ID;
        header.prog_if = T::PROG_IF;
        header.subclass = T::SUBCLASS;
        header.class_code = T::CLASS_CODE;
        header.header_type = header_type;
        T::init_capability(&mut cfg);

        cfg
    }

    pub fn as_common_header(&self) -> &HeaderCommon {
        self.as_header::<HeaderCommon>()
    }

    pub fn as_common_header_mut(&mut self) -> &mut HeaderCommon {
        self.as_header_mut::<HeaderCommon>()
    }

    pub fn as_header<T>(&self) -> &T
    where
        T: FromBytes + KnownLayout + Immutable,
    {
        T::ref_from_bytes(&self.buf[0..size_of::<T>()]).unwrap()
    }

    pub fn as_header_mut<T>(&mut self) -> &mut T
    where
        T: IntoBytes + FromBytes + KnownLayout + Immutable,
    {
        T::mut_from_bytes(&mut self.buf[0..size_of::<T>()]).unwrap()
    }

    pub fn read(&self, offset: u16, buf: &mut [u8]) {
        buf.copy_from_slice(&self.buf[offset as usize..offset as usize + buf.len()]);
    }

    pub fn write(&mut self, offset: u16, buf: &[u8]) {
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
