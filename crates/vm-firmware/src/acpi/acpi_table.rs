use vm_mm::manager::MemoryAddressSpace;

use crate::acpi::error::AcpiError;
use crate::acpi::r#type::dsdt::Dsdt;
use crate::acpi::r#type::fadt::Fadt;
use crate::acpi::r#type::madt::Madt;
use crate::acpi::r#type::mcfg::Mcfg;
use crate::acpi::r#type::rsdp::Rsdp;
use crate::acpi::r#type::xsdt::Xsdt;

pub fn get_address(len: usize) -> u64 {
    todo!()
}

fn reserve_address(hint_address: u64, len: usize) -> u64 {
    todo!()
}

pub struct AcpiTable {
    pub(crate) definition_block: Vec<u8>,
    pub(crate) apic_base_address: u32,
    pub(crate) interrupt_controllers: Vec<u8>,
}

impl AcpiTable {
    pub fn install(
        self,
        guest_memory_allocator: impl FnMut(usize) -> Option<u64>,
        memory: &MemoryAddressSpace,
        hint_rsdp_address: u64,
    ) -> Result<(), AcpiError> {
        reserve_address(hint_rsdp_address, size_of::<Rsdp>());

        let dsdt = Dsdt::new(self.definition_block);
        let dsdt_address = dsdt.install(memory)?;

        let fadt = Fadt::new(dsdt_address);
        let fadt_address = fadt.install(memory)?;

        let madt = Madt::new(self.apic_base_address, self.interrupt_controllers);
        let madt_address = madt.install(memory)?;

        let mcfg = Mcfg::new(todo!());
        let mcfg_address = mcfg.install(memory)?;

        let xsdt = Xsdt::new(vec![fadt_address, madt_address, mcfg_address]);
        let xsdt_address = xsdt.install(memory)?;

        let rsdp = Rsdp::new(xsdt_address);
        rsdp.install(memory, hint_rsdp_address)?;

        Ok(())
    }
}
