use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

use crate::device::function::PciTypeFunctionCommon;
use crate::error::Error;
use crate::types::configuration_space::capability::ExtendedCapability;
use crate::types::configuration_space::capability::StandardCapability;
use crate::types::configuration_space::header::CommonHeaderOffset;
use crate::types::configuration_space::header::HeaderCommon;
use crate::types::configuration_space::status::PciStatus;

pub mod capability;

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

    pub fn alloc_capability(&mut self, cap: StandardCapability) -> Result<u8, Error> {
        let header = self.as_common_header_mut();
        header.status |= PciStatus::CapList as u16;

        let offset = self.next_available_capability_pointer;
        let offset = offset.next_multiple_of(4);

        let cap_len = cap.cap_len();
        let cap_len = u16::try_from(cap_len).map_err(|_| Error::CapTooLarge)?;

        if offset as u16 + cap_len > 0x100 {
            return Err(Error::CapNoSpace);
        }

        self.buf[self.last_capability_next_pointer as usize] = offset as u8;
        self.last_capability_next_pointer = offset as u8 + 1;
        self.next_available_capability_pointer = offset + cap_len as u16;

        let new_cap = &mut self.buf[(offset as usize)..((offset + cap_len as u16) as usize)];
        new_cap[0] = cap.cap_id;
        new_cap[1] = 0;
        new_cap[2..].copy_from_slice(&cap.data);

        Ok(offset as u8)
    }

    pub fn alloc_ext_capability(&mut self, cap: ExtendedCapability) -> Result<u16, Error> {
        let offset = self.next_available_ext_capability_pointer;
        let offset = offset.next_multiple_of(4);

        let cap_len = cap.cap_len();
        let cap_len = u16::try_from(cap_len).map_err(|_| Error::CapTooLarge)?;

        if offset as u16 + cap_len > 4096 {
            return Err(Error::CapNoSpace);
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
        new_cap[0..2].copy_from_slice(&cap.cap_id.to_le_bytes());
        new_cap[2..4].copy_from_slice(&cap.next_or_ver.to_le_bytes());
        new_cap[4..].copy_from_slice(&cap.data);

        Ok(offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::configuration_space::capability::ExtendedCapability;
    use crate::types::configuration_space::capability::StandardCapability;

    #[test]
    fn test_standard_capability_allocation() -> Result<(), Error> {
        let mut cfg = ConfigurationSpace::new();

        let first_cap_offset;
        let first_cap_len;
        {
            let cap_id = 0x05;
            let cap_data = vec![0xAA, 0xBB, 0xCC];
            let cap = StandardCapability::new(cap_id, cap_data.clone());

            first_cap_len = cap.cap_len();
            first_cap_offset = cfg.alloc_capability(cap)?;

            let header = cfg.as_common_header_mut();
            assert!(header.status & (PciStatus::CapList as u16) != 0);

            let offset = first_cap_offset as usize;
            assert_eq!(first_cap_len, cap_data.len() + 2);
            assert_eq!(offset, CommonHeaderOffset::CapabilityStart as usize);
            assert_eq!(cfg.buf[offset], cap_id);
            assert_eq!(cfg.buf[offset + 1], 0);
            assert_eq!(&cfg.buf[offset + 2..offset + 5], &cap_data[..]);
        }

        let second_cap_offset;
        let second_cap_len;
        {
            let cap_id = 0x06;
            let cap_data = vec![0xDD, 0xEE, 0xFF];
            let cap = StandardCapability::new(cap_id, cap_data.clone());

            second_cap_len = cap.cap_len();
            second_cap_offset = cfg.alloc_capability(cap)?;

            let offset = second_cap_offset as usize;
            assert_eq!(
                offset,
                (first_cap_offset as usize + first_cap_len).next_multiple_of(4)
            );
            assert_eq!(cfg.buf[first_cap_offset as usize + 1], offset as u8);
            assert_eq!(cfg.buf[offset], cap_id);
            assert_eq!(cfg.buf[offset + 1], 0);
            assert_eq!(&cfg.buf[offset + 2..offset + 5], &cap_data[..]);
        }

        {
            let cap_id = 0x07;
            let cap_data = vec![0x11, 0x22, 0x33];
            let cap = StandardCapability::new(cap_id, cap_data.clone());

            let offset = cfg.alloc_capability(cap)?;

            let offset = offset as usize;
            assert_eq!(
                offset,
                (second_cap_offset as usize + second_cap_len).next_multiple_of(4)
            );
            assert_eq!(cfg.buf[second_cap_offset as usize + 1], offset as u8);
            assert_eq!(cfg.buf[offset], cap_id);
            assert_eq!(cfg.buf[offset + 1], 0);
            assert_eq!(&cfg.buf[offset + 2..offset + 5], &cap_data[..]);
        }

        {
            let cap_id = 0x07;
            let cap = StandardCapability::new(cap_id, [0; 256].to_vec());

            assert!(cfg.alloc_capability(cap).is_err());
        }

        Ok(())
    }

    #[test]
    fn test_extended_capability_allocation() -> Result<(), Error> {
        let mut cfg = ConfigurationSpace::new();

        let cap_id1 = 0x1234;
        let cap_version1 = 1;
        let cap1_data = vec![0x11, 0x22, 0x33];
        let cap1 = ExtendedCapability::new(cap_id1, cap_version1, cap1_data.clone());
        let first_cap_offset = cfg.alloc_ext_capability(cap1.clone())?;

        let cap_id2 = 0x5678;
        let cap_version2 = 2;
        let cap2_data = vec![0x44, 0x55, 0x66];
        let cap2 = ExtendedCapability::new(cap_id2, cap_version2, cap2_data.clone());
        let second_cap_offset = cfg.alloc_ext_capability(cap2.clone())?;

        let cap_id3 = 0xabcd;
        let cap_version3 = 3;
        let cap3_data = vec![0x77, 0x88, 0x99];
        let cap3 = ExtendedCapability::new(cap_id3, cap_version3, cap3_data.clone());
        let third_cap_offset = cfg.alloc_ext_capability(cap3)?;

        {
            assert_eq!(
                first_cap_offset,
                CommonHeaderOffset::ExtendedCapabilityStart as u16
            );
            let first_cap_offset = first_cap_offset as usize;
            assert_eq!(
                u16::from_le_bytes(
                    cfg.buf[first_cap_offset..first_cap_offset + 2]
                        .try_into()
                        .unwrap()
                ),
                cap_id1
            );
            assert_eq!(
                u16::from_le_bytes(
                    cfg.buf[first_cap_offset + 2..first_cap_offset + 4]
                        .try_into()
                        .unwrap()
                ),
                second_cap_offset << 4 | cap_version1
            );
            assert_eq!(
                cfg.buf[first_cap_offset + 4..first_cap_offset + 4 + cap1_data.len()],
                cap1_data
            );
        }

        {
            assert_eq!(
                second_cap_offset,
                (first_cap_offset + cap1.cap_len() as u16).next_multiple_of(4)
            );
            let second_cap_offset = second_cap_offset as usize;
            assert_eq!(
                u16::from_le_bytes(
                    cfg.buf[second_cap_offset..second_cap_offset + 2]
                        .try_into()
                        .unwrap()
                ),
                cap_id2
            );
            assert_eq!(
                u16::from_le_bytes(
                    cfg.buf[second_cap_offset + 2..second_cap_offset + 4]
                        .try_into()
                        .unwrap()
                ),
                third_cap_offset << 4 | cap_version2
            );
            assert_eq!(
                cfg.buf[second_cap_offset + 4..second_cap_offset + 4 + cap2_data.len()],
                cap2_data
            );
        }

        {
            assert_eq!(
                third_cap_offset,
                (second_cap_offset + cap2.cap_len() as u16).next_multiple_of(4)
            );
            let third_cap_offset = third_cap_offset as usize;
            assert_eq!(
                u16::from_le_bytes(
                    cfg.buf[third_cap_offset..third_cap_offset + 2]
                        .try_into()
                        .unwrap()
                ),
                cap_id3
            );
            assert_eq!(
                u16::from_le_bytes(
                    cfg.buf[third_cap_offset + 2..third_cap_offset + 4]
                        .try_into()
                        .unwrap()
                ),
                cap_version3
            );
            assert_eq!(
                cfg.buf[third_cap_offset + 4..third_cap_offset + 4 + cap3_data.len()],
                cap3_data
            );
        }

        Ok(())
    }
}
