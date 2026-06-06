use zerocopy::IntoBytes;

use crate::apic::CREATOR_ID;
use crate::apic::CREATOR_REVISION;
use crate::apic::OEM_REVISION;
use crate::apic::OEM_TABLE_ID;
use crate::apic::OEMID;
use crate::apic::r#type::common_header::CommonHeader;
use crate::apic::utils::checksum;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dsdt() {
        let definition_block = vec![0x0, 0x1, 0x2, 0x3];
        let dsdt = Dsdt::new(definition_block.clone());

        assert_eq!(
            checksum(&[dsdt.header.as_bytes(), &definition_block].concat()),
            0
        );
        let length = dsdt.header.length;
        assert_eq!(
            length,
            (size_of::<CommonHeader>() + definition_block.len())
                .try_into()
                .unwrap()
        );
    }
}
