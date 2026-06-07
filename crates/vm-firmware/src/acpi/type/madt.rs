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

/// Multiple APIC Description Table
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

    pub fn len(&self) -> usize {
        self.header.length as usize
    }

    pub fn install(&self, memory: &MemoryAddressSpace) -> Result<u64, AcpiError> {
        let address = get_address(self.len());
        memory
            .copy_from_slice(
                address,
                &[
                    self.header.as_bytes(),
                    self.local_interrupt_controller_address.as_bytes(),
                    self.flags.as_bytes(),
                    self.interrupt_controllers.as_bytes(),
                ]
                .concat(),
            )
            .map_err(|_| AcpiError::CopyToMemory)?;

        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_madt() {
        let madt = Madt::new(0xdeadbeef, vec![0x0, 0x1, 0x2, 0x3]);

        assert_eq!(
            checksum(
                &[
                    madt.header.as_bytes(),
                    madt.local_interrupt_controller_address.as_bytes(),
                    madt.flags.as_bytes(),
                    madt.interrupt_controllers.as_bytes(),
                ]
                .concat(),
            ),
            0
        );
        assert_eq!(
            madt.len(),
            (size_of::<CommonHeader>() + size_of::<u32>() * 2 + madt.interrupt_controllers.len())
                .try_into()
                .unwrap()
        );
    }
}
