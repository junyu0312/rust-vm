use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;

use crate::acpi::error::AcpiError;
use crate::acpi::r#type::dsdt::Dsdt;
use crate::acpi::r#type::fadt::Fadt;
use crate::acpi::r#type::madt::Madt;
use crate::acpi::r#type::mcfg::Mcfg;
use crate::acpi::r#type::mcfg::PciRangeEntry;
use crate::acpi::r#type::rsdp::Rsdp;
use crate::acpi::r#type::xsdt::Xsdt;

pub struct AcpiTable {
    pub(crate) definition_block: Vec<u8>,
    pub(crate) apic_base_address: u32,
    pub(crate) interrupt_controllers: Vec<u8>,
    pub(crate) pci_range_entry: PciRangeEntry, // We only support one yet
}

impl AcpiTable {
    pub fn install(
        self,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        rsdp_address: u64,
    ) -> Result<(), AcpiError> {
        ram_allocator.reserve(rsdp_address, size_of::<Rsdp>())?;

        let dsdt = Dsdt::new(self.definition_block);
        let dsdt_address = dsdt.install(ram_allocator, memory)?;

        let fadt = Fadt::new(dsdt_address);
        let fadt_address = fadt.install(ram_allocator, memory)?;

        let madt = Madt::new(self.apic_base_address, self.interrupt_controllers);
        let madt_address = madt.install(ram_allocator, memory)?;

        let mcfg = Mcfg::new(vec![self.pci_range_entry]);
        let mcfg_address = mcfg.install(ram_allocator, memory)?;

        let xsdt = Xsdt::new(vec![fadt_address, madt_address, mcfg_address]);
        let xsdt_address = xsdt.install(ram_allocator, memory)?;

        let rsdp = Rsdp::new(xsdt_address);
        rsdp.install(memory, rsdp_address)?;

        Ok(())
    }
}
