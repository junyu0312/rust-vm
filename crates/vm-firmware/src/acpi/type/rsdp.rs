use vm_mm::manager::MemoryAddressSpace;
use zerocopy::Immutable;
use zerocopy::IntoBytes;

use crate::acpi::OEMID;
use crate::acpi::error::AcpiError;
use crate::acpi::utils::checksum;

/// Root System Description Pointer
#[derive(Immutable, IntoBytes)]
#[repr(C, packed)]
pub struct Rsdp {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_addr: u32,
    length: u32,
    xsdt_addr: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

impl Rsdp {
    pub fn new(xsdt_addr: u64) -> Rsdp {
        let mut raw = Rsdp {
            signature: *b"RSD PTR ",
            checksum: 0,
            oem_id: OEMID,
            revision: 2,
            rsdt_addr: 0, // ignored, using xsdt in revision 2
            length: size_of::<Rsdp>().try_into().unwrap(),
            xsdt_addr,
            extended_checksum: 0,
            reserved: [0; 3],
        };

        raw.checksum = checksum(&raw.as_bytes()[0..20]);

        // This is a checksum of the entire table, including both checksum fields.
        raw.extended_checksum = checksum(raw.as_bytes());

        raw
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.length as usize
    }

    pub fn install(&self, memory: &MemoryAddressSpace, rsdp_address: u64) -> Result<(), AcpiError> {
        memory.copy_from_slice(rsdp_address, self.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsdp() {
        let rsdp = Rsdp::new(0xdeadbeef);

        assert_eq!(checksum(&rsdp.as_bytes()[0..20]), 0);
        assert_eq!(checksum(rsdp.as_bytes()), 0);
        assert_eq!(rsdp.len(), size_of::<Rsdp>());
    }
}
