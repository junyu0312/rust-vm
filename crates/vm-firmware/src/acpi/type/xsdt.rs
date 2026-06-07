use vm_mm::manager::MemoryAddressSpace;
use zerocopy::IntoBytes;

use crate::acpi::CREATOR_ID;
use crate::acpi::CREATOR_REVISION;
use crate::acpi::OEM_REVISION;
use crate::acpi::OEM_TABLE_ID;
use crate::acpi::OEMID;
use crate::acpi::acpi_table::get_address;
use crate::acpi::error::AcpiError;
use crate::acpi::r#type::common_header::CommonHeader;
use crate::acpi::utils::checksum;

/// Extended System Description Table
pub struct Xsdt {
    header: CommonHeader,
    entry: Vec<u64>,
}

impl Xsdt {
    pub fn new(entry: Vec<u64>) -> Self {
        let length = size_of::<CommonHeader>() + entry.as_bytes().len();

        let mut raw = Xsdt {
            header: CommonHeader {
                signature: *b"XSDT",
                length: length.try_into().unwrap(),
                revision: 1,
                checksum: 0,
                oem_id: OEMID,
                oem_table_id: OEM_TABLE_ID,
                oem_revision: OEM_REVISION,
                creator_id: CREATOR_ID,
                creator_revision: CREATOR_REVISION,
            },
            entry,
        };

        raw.header.checksum = checksum(&[raw.header.as_bytes(), raw.entry.as_bytes()].concat());

        raw
    }

    pub fn len(&self) -> usize {
        self.header.length as usize
    }

    pub fn install(&self, memory: &MemoryAddressSpace) -> Result<u64, AcpiError> {
        let address = get_address(self.len());
        memory
            .copy_from_slice(
                address,
                &[self.header.as_bytes(), self.entry.as_bytes()].concat(),
            )
            .map_err(|_| AcpiError::CopyToMemory)?;

        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xsdt() {
        let xsdt = Xsdt::new(vec![0x00000000, 0x11111111]);

        assert_eq!(
            checksum(&[xsdt.header.as_bytes(), xsdt.entry.as_bytes()].concat()),
            0
        );
        assert_eq!(
            xsdt.len(),
            (size_of::<CommonHeader>() + xsdt.entry.as_bytes().len())
                .try_into()
                .unwrap()
        );
    }
}
