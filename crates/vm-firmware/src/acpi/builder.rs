use std::cell::OnceCell;

use crate::acpi::acpi_table::AcpiTable;
use crate::acpi::error::AcpiError;
use crate::acpi::r#type::mcfg::PciRangeEntry;

#[derive(Default)]
pub struct AcpiTableBuilder {
    vcpus: OnceCell<u8>,
    definition_block: OnceCell<Vec<u8>>,
    apic_base_address: OnceCell<u32>,
    pci_mmio_base_addr: OnceCell<u64>,

    #[cfg(target_arch = "x86_64")]
    io_apic_address: OnceCell<u32>,
}

impl AcpiTableBuilder {
    pub fn set_vcpus(self, vcpus: u8) -> Result<AcpiTableBuilder, AcpiError> {
        self.vcpus
            .set(vcpus)
            .map_err(|_| AcpiError::FieldAlreadySet("vcpus"))?;

        Ok(self)
    }

    pub fn set_definition_block(
        self,
        definition_block: Vec<u8>,
    ) -> Result<AcpiTableBuilder, AcpiError> {
        self.definition_block
            .set(definition_block)
            .map_err(|_| AcpiError::FieldAlreadySet("definition_block"))?;

        Ok(self)
    }

    pub fn set_apic_base_address(
        self,
        apic_base_address: u32,
    ) -> Result<AcpiTableBuilder, AcpiError> {
        self.apic_base_address
            .set(apic_base_address)
            .map_err(|_| AcpiError::FieldAlreadySet("apic_base_address"))?;

        Ok(self)
    }

    #[cfg(target_arch = "x86_64")]
    pub fn set_io_apic_address(self, io_apic_address: u32) -> Result<AcpiTableBuilder, AcpiError> {
        self.io_apic_address
            .set(io_apic_address)
            .map_err(|_| AcpiError::FieldAlreadySet("io_apic_address"))?;

        Ok(self)
    }

    pub fn set_pci_mmio_base_addr(self, base_address: u64) -> Result<AcpiTableBuilder, AcpiError> {
        self.pci_mmio_base_addr
            .set(base_address)
            .map_err(|_| AcpiError::FieldAlreadySet("set_pci_mmio_base_addr"))?;

        Ok(self)
    }

    pub fn build(mut self) -> Result<AcpiTable, AcpiError> {
        let interrupt_controllers = self.setup_arch_interrupt_controllers()?;
        let pci_mmio_base_addr = self
            .pci_mmio_base_addr
            .take()
            .ok_or_else(|| AcpiError::FieldNotSet("pci_mmio_configuration_space"))?;

        let table = AcpiTable {
            definition_block: self
                .definition_block
                .take()
                .ok_or_else(|| AcpiError::FieldNotSet("definition_block"))?,
            apic_base_address: self
                .apic_base_address
                .take()
                .ok_or_else(|| AcpiError::FieldNotSet("apic_base_address"))?,
            interrupt_controllers,
            pci_range_entry: PciRangeEntry::new(pci_mmio_base_addr, 0, 0, 0),
        };

        Ok(table)
    }

    #[cfg(target_arch = "x86_64")]
    fn setup_arch_interrupt_controllers(&mut self) -> Result<Vec<u8>, AcpiError> {
        use zerocopy::IntoBytes;

        use crate::acpi::r#type::arch::x86_64::IoApic;
        use crate::acpi::r#type::arch::x86_64::LocalApic;

        let vcpus = *self
            .vcpus
            .get()
            .ok_or_else(|| AcpiError::FieldNotSet("vcpus"))?;

        let mut buf =
            Vec::with_capacity(size_of::<IoApic>() + size_of::<LocalApic>() * vcpus as usize);

        {
            let io_apic = IoApic::new(
                *self
                    .io_apic_address
                    .get()
                    .ok_or_else(|| AcpiError::FieldNotSet("io_apic_address"))?,
            );
            buf.extend_from_slice(io_apic.as_bytes());
        }

        {
            for vcpu in 0..vcpus {
                let local_apic = LocalApic::new(vcpu);
                buf.extend_from_slice(local_apic.as_bytes());
            }
        }

        Ok(buf)
    }

    #[cfg(target_arch = "aarch64")]
    fn setup_arch_interrupt_controllers(&mut self) -> Result<Vec<u8>, AcpiError> {
        Err(AcpiError::ArchNotSupport("aarch64"))
    }
}
