use zerocopy::IntoBytes;

use crate::apic::CREATOR_ID;
use crate::apic::CREATOR_REVISION;
use crate::apic::OEM_REVISION;
use crate::apic::OEM_TABLE_ID;
use crate::apic::OEMID;
use crate::apic::r#type::common_header::CommonHeader;
use crate::apic::utils::checksum;

/// Multiple APIC Description Table
#[repr(C, packed)]
pub struct Madt {
    header: CommonHeader,
    local_interrupt_controller_address: u32,
    flags: u32,
    interrupt_controllers: Vec<u8>,
}

impl Madt {
    pub fn new(local_interrupt_controller_address: u32, interrupt_controllers: Vec<u8>) -> Self {
        let length = size_of::<CommonHeader>()
            + size_of::<u32>()
            + size_of::<u32>()
            + interrupt_controllers.len();

        let mut raw = Madt {
            header: CommonHeader {
                signature: *b"APIC",
                length: length.try_into().unwrap(),
                revision: 6,
                checksum: 0,
                oem_id: OEMID,
                oem_table_id: OEM_TABLE_ID,
                oem_revision: OEM_REVISION,
                creator_id: CREATOR_ID,
                creator_revision: CREATOR_REVISION,
            },
            local_interrupt_controller_address,
            // `A one indicates that the system also has a PC-AT-compatible dual-8259 setup.
            // The 8259 vectors must be disabled (that is, masked) when enabling the ACPI APIC operation.`
            // TODO: 0 or 1?
            flags: 0,
            interrupt_controllers: interrupt_controllers.clone(),
        };

        let flags = raw.flags;
        raw.header.checksum = checksum(
            &[
                raw.header.as_bytes(),
                local_interrupt_controller_address.as_bytes(),
                flags.as_bytes(),
                interrupt_controllers.as_bytes(),
            ]
            .concat(),
        );

        raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_madt() {
        let local_interrupt_controller_address = 0xdeadbeef;
        let interrupt_controllers = vec![0x0, 0x1, 0x2, 0x3];
        let madt = Madt::new(
            local_interrupt_controller_address,
            interrupt_controllers.clone(),
        );

        let header = madt.header;
        let flags = madt.flags;
        assert_eq!(
            checksum(
                &[
                    header.as_bytes(),
                    local_interrupt_controller_address.as_bytes(),
                    flags.as_bytes(),
                    interrupt_controllers.as_bytes(),
                ]
                .concat(),
            ),
            0
        );
        let length = header.length;
        assert_eq!(
            length,
            (size_of::<CommonHeader>() + size_of::<u32>() * 2 + interrupt_controllers.len())
                .try_into()
                .unwrap()
        );
    }
}
