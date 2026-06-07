use std::cell::OnceCell;

use crate::acpi::acpi_table::AcpiTable;
use crate::acpi::error::AcpiError;

#[derive(Default)]
pub struct AcpiTableBuilder {
    apic_base_address: OnceCell<u32>,
}

impl AcpiTableBuilder {
    pub fn build(mut self) -> Result<AcpiTable, AcpiError> {
        let table = AcpiTable {
            apic_base_address: self
                .apic_base_address
                .take()
                .ok_or_else(|| AcpiError::FieldNotSet("acpi_base_address"))?,
        };

        Ok(table)
    }
}
