use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;
use zerocopy::Immutable;
use zerocopy::IntoBytes;

use crate::acpi::CREATOR_ID;
use crate::acpi::CREATOR_REVISION;
use crate::acpi::HYPERVISOR_VENDOR_ID;
use crate::acpi::OEM_REVISION;
use crate::acpi::OEM_TABLE_ID;
use crate::acpi::OEMID;
use crate::acpi::error::AcpiError;
use crate::acpi::r#type::common_header::CommonHeader;
use crate::acpi::r#type::generic_address_structure_format::GenericAddressStructureFormat;
use crate::acpi::utils::checksum;

#[derive(Default, Immutable, IntoBytes)]
#[repr(C, packed)]
pub struct Fadt {
    header: CommonHeader,
    firmware_ctrl: u32,
    dsdt: u32,
    reserved_0: u8,
    preferred_pm_profile: u8,
    sci_int: u16,
    smi_cmd: u32,
    acpi_enable: u8,
    acpi_disable: u8,
    s4bios_req: u8,
    pstate_cnt: u8,
    pm1a_evt_blk: u32,
    pm1b_evt_blk: u32,
    pm1a_cnt_blk: u32,
    pm1b_cnt_blk: u32,
    pm2_cnt_blk: u32,
    pm_tmr_blk: u32,
    gpe0_blk: u32,
    gpe1_blk: u32,
    pm1_evt_len: u8,
    pm1_cnt_len: u8,
    pm2_cnt_len: u8,
    pm_tmr_len: u8,
    gpe0_blk_len: u8,
    gpe1_blk_len: u8,
    gpe1_base: u8,
    cst_cnt: u8,
    p_lvl2_lat: u16,
    p_lvl3_lat: u16,
    flush_size: u16,
    flush_stride: u16,
    duty_offset: u8,
    duty_width: u8,
    day_alrm: u8,
    mon_alrm: u8,
    century: u8,
    iapc_boot_arch: u16,
    reserved_1: u8,
    flags: u32,
    reset_reg: GenericAddressStructureFormat,
    reset_value: u8,
    arm_boot_arch: u16,
    fadt_minor_version: u8,
    x_firmware_ctrl: u64,
    x_dsdt: u64,
    x_pm1a_evt_blk: GenericAddressStructureFormat,
    x_pm1b_evt_blk: GenericAddressStructureFormat,
    x_pm1a_cnt_blk: GenericAddressStructureFormat,
    x_pm1b_cnt_blk: GenericAddressStructureFormat,
    x_pm2_cnt_blk: GenericAddressStructureFormat,
    x_pm_tmr_blk: GenericAddressStructureFormat,
    x_gpe0_blk: GenericAddressStructureFormat,
    x_gpe1_blk: GenericAddressStructureFormat,
    sleep_control_reg: GenericAddressStructureFormat,
    sleep_status_reg: GenericAddressStructureFormat,
    hypervisor_vendor_id: [u8; 8],
}

impl Fadt {
    pub fn new(x_dsdt: u64) -> Self {
        let mut raw = Fadt {
            header: CommonHeader {
                signature: *b"FACP",
                length: size_of::<Fadt>().try_into().unwrap(),
                revision: 6,
                checksum: 0,
                oem_id: OEMID,
                oem_table_id: OEM_TABLE_ID,
                oem_revision: OEM_REVISION,
                creator_id: CREATOR_ID,
                creator_revision: CREATOR_REVISION,
            },
            flags: 0,
            fadt_minor_version: 5, // ACPI 6.6 specification says it is 5.
            x_dsdt,
            hypervisor_vendor_id: HYPERVISOR_VENDOR_ID,
            // TODO
            pm1a_cnt_blk: 0x1000,
            // TODO
            pm1_evt_len: 16,
            // TODO
            pm1a_evt_blk: 0x1004,
            // TODO
            pm1_cnt_len: 32,
            // TODO
            sci_int: 9,
            ..Default::default()
        };

        raw.header.checksum = checksum(raw.as_bytes());

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
        memory.copy_from_slice(address, self.as_bytes())?;

        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fadt() {
        let fadt = Fadt::new(0x12345678);

        assert_eq!(checksum(fadt.as_bytes()), 0);
        assert_eq!(fadt.len(), fadt.as_bytes().len());
    }
}
