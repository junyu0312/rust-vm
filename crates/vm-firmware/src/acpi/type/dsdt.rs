use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;
use zerocopy::IntoBytes;

use crate::acpi::CREATOR_ID;
use crate::acpi::CREATOR_REVISION;
use crate::acpi::OEM_REVISION;
use crate::acpi::OEM_TABLE_ID;
use crate::acpi::OEMID;
use crate::acpi::error::AcpiError;
use crate::acpi::r#type::common_header::CommonHeader;
use crate::acpi::utils::checksum;

/// Differentiated System Description Table
pub struct Dsdt {
    header: CommonHeader,
    definition_block: Vec<u8>,
}

impl Dsdt {
    pub fn new(definition_block: Vec<u8>) -> Self {
        let length = size_of::<CommonHeader>() + definition_block.len();

        let mut raw = Dsdt {
            header: CommonHeader {
                signature: *b"DSDT",
                length: 0,
                revision: 2,
                checksum: 0,
                oem_id: OEMID,
                oem_table_id: OEM_TABLE_ID,
                oem_revision: OEM_REVISION,
                creator_id: CREATOR_ID,
                creator_revision: CREATOR_REVISION,
            },
            definition_block,
        };

        raw.header.length = length.try_into().unwrap();
        raw.header.checksum = checksum(&[raw.header.as_bytes(), &raw.definition_block].concat());

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
        let address = ram_allocator.alloc(self.len())?.start;
        memory.copy_from_slice(
            address,
            &[self.header.as_bytes(), self.definition_block.as_bytes()].concat(),
        )?;

        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dsdt() {
        let dsdt = Dsdt::new(vec![0x0, 0x1, 0x2, 0x3]);

        assert_eq!(
            checksum(&[dsdt.header.as_bytes(), &dsdt.definition_block].concat()),
            0
        );
        assert_eq!(
            dsdt.len(),
            size_of::<CommonHeader>() + dsdt.definition_block.len()
        );
    }
}
