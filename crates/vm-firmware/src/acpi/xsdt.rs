// use zerocopy::IntoBytes;

use zerocopy::IntoBytes;

use crate::acpi::CREATOR_ID;
use crate::acpi::CREATOR_REVISION;
use crate::acpi::OEM_REVISION;
use crate::acpi::OEM_TABLE_ID;
use crate::acpi::OEMID;
use crate::acpi::r#type::common_header::CommonHeader;
use crate::acpi::utils::checksum;

/// Extended System Description Table
#[repr(C, packed)]
pub struct Xsdt {
    header: CommonHeader,
    entry: Vec<u64>,
}

impl Xsdt {
    pub fn new(entry: Vec<u64>) -> Self {
        let mut raw = Xsdt {
            header: CommonHeader {
                signature: *b"XSDT",
                length: (size_of::<CommonHeader>() + entry.as_bytes().len())
                    .try_into()
                    .unwrap(),
                revision: 1,
                checksum: 0,
                oem_id: OEMID,
                oem_table_id: OEM_TABLE_ID,
                oem_revision: OEM_REVISION,
                creator_id: CREATOR_ID,
                creator_revision: CREATOR_REVISION,
            },
            entry: entry.clone(),
        };

        raw.header.checksum = checksum(&[raw.header.as_bytes(), entry.as_bytes()].concat());

        raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xsdt() {
        let rsdp = Xsdt::new(vec![0x00000000, 0x11111111]);
        let header = rsdp.header;
        let entry = rsdp.entry;

        assert_eq!(checksum(&[header.as_bytes(), entry.as_bytes()].concat()), 0);
    }
}
