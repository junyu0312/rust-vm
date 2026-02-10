use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

use crate::types::configuration_space::common::CommonHeaderOffset;
use crate::types::configuration_space::common::HeaderCommon;
use crate::types::configuration_space::common::status::PciStatus;

pub mod capability;
pub mod common;
pub mod type0;
pub mod type1;

const FIRST_CAPABILITY_OFFSET: u8 = 0x40;

pub struct ConfigurationSpace {
    buf: [u8; 4096],
    next_capability_pointer: u8,
    next_available_capability_pointer: u8,
}

impl ConfigurationSpace {
    pub fn new(buf: [u8; 4096]) -> Self {
        ConfigurationSpace {
            buf,
            next_capability_pointer: CommonHeaderOffset::CapabilityPointer as u8,
            next_available_capability_pointer: FIRST_CAPABILITY_OFFSET,
        }
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

    /// len: the whole length of the capability
    pub fn alloc_capability(&mut self, cap_id: u8, cap_len: u8) -> &mut [u8] {
        let header = self.as_common_header_mut();
        header.status |= PciStatus::PciStatusCapList as u16;

        let offset = self.next_available_capability_pointer;

        self.buf[self.next_capability_pointer as usize] = offset;
        self.next_capability_pointer = offset + 1;
        self.next_available_capability_pointer = offset + cap_len;

        let cap = &mut self.buf[(offset as usize)..((offset + cap_len) as usize)];
        cap[0] = cap_id;
        cap[1] = 0; // next

        cap
    }
}
