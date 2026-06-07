use std::cell::OnceCell;

use crate::acpi::acpi_table::AcpiTable;
use crate::acpi::error::AcpiError;

#[derive(Default)]
pub struct AcpiTableBuilder {
    vcpus: OnceCell<u8>,
    apic_base_address: OnceCell<u32>,

    #[cfg(target_arch = "x86_64")]
    io_apic_address: OnceCell<u32>,
}

impl AcpiTableBuilder {
    pub fn set_vcpus(&mut self, vcpus: u8) -> Result<(), AcpiError> {
        self.vcpus
            .set(vcpus)
            .map_err(|_| AcpiError::FieldAlreadySet("vcpus"))?;

        Ok(())
    }

    pub fn set_apic_base_address(&mut self, apic_base_address: u32) -> Result<(), AcpiError> {
        self.apic_base_address
            .set(apic_base_address)
            .map_err(|_| AcpiError::FieldAlreadySet("apic_base_address"))?;

        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    pub fn set_io_apic_address(&mut self, io_apic_address: u32) -> Result<(), AcpiError> {
        self.io_apic_address
            .set(io_apic_address)
            .map_err(|_| AcpiError::FieldAlreadySet("io_apic_address"))?;

        Ok(())
    }

    pub fn build(mut self) -> Result<AcpiTable, AcpiError> {
        let interrupt_controllers = self.setup_arch_interrupt_controllers()?;

        let table = AcpiTable {
            apic_base_address: self
                .apic_base_address
                .take()
                .ok_or_else(|| AcpiError::FieldNotSet("acpi_base_address"))?,
            interrupt_controllers,
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
