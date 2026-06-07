use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;
use zerocopy::Immutable;
use zerocopy::IntoBytes;

use crate::acpi::CREATOR_ID;
use crate::acpi::CREATOR_REVISION;
use crate::acpi::OEM_REVISION;
use crate::acpi::OEM_TABLE_ID;
use crate::acpi::OEMID;
use crate::acpi::error::AcpiError;
use crate::acpi::r#type::common_header::CommonHeader;
use crate::acpi::utils::checksum;

#[derive(Clone, Immutable, IntoBytes)]
#[repr(C, packed)]
pub struct PciRangeEntry {
    base_address: u64,
    segment: u16,
    start: u8,
    end: u8,
    reserved: u32,
}

impl PciRangeEntry {
    pub fn new(base_address: u64, segment: u16, start: u8, end: u8) -> Self {
        PciRangeEntry {
            base_address,
            segment,
            start,
            end,
            reserved: 0,
        }
    }
}

pub struct Mcfg {
    header: CommonHeader,
    reserved: u64,
    entry: Vec<PciRangeEntry>,
}

impl Mcfg {
    pub fn new(entry: Vec<PciRangeEntry>) -> Self {
        let length = (size_of::<CommonHeader>() + size_of::<u64>() + entry.as_bytes().len())
            .try_into()
            .unwrap();

        let mut raw = Mcfg {
            header: CommonHeader {
                signature: *b"MCFG",
                length,
                revision: 1,
                checksum: 0,
                oem_id: OEMID,
                oem_table_id: OEM_TABLE_ID,
                oem_revision: OEM_REVISION,
                creator_id: CREATOR_ID,
                creator_revision: CREATOR_REVISION,
            },
            reserved: 0,
            entry,
        };

        raw.header.checksum = checksum(
            &[
                raw.header.as_bytes(),
                raw.reserved.as_bytes(),
                raw.entry.as_bytes(),
            ]
            .concat(),
        );

        raw
    }

    pub fn len(&self) -> usize {
        self.header.length as usize
    }

    pub fn install(
        &self,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
    ) -> Result<u64, AcpiError> {
        let address = ram_allocator.alloc(self.len())?;
        memory.copy_from_slice(
            address,
            &[
                self.header.as_bytes(),
                self.reserved.as_bytes(),
                self.entry.as_bytes(),
            ]
            .concat(),
        )?;

        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcfg() {
        let entry = vec![PciRangeEntry::new(0xdeadbeef, 0, 0, 0)];
        let mcfg = Mcfg::new(entry.clone());

        assert_eq!(
            checksum(
                &[
                    mcfg.header.as_bytes(),
                    mcfg.reserved.as_bytes(),
                    mcfg.entry.as_bytes(),
                ]
                .concat()
            ),
            0
        );
        assert_eq!(
            mcfg.len(),
            size_of::<CommonHeader>() + size_of::<u64>() + entry.as_bytes().len()
        );
    }
}
