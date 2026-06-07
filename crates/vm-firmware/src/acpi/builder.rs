use crate::acpi::acpi_table::AcpiTable;

#[derive(Default)]
pub struct AcpiTableBuilder {}

impl AcpiTableBuilder {
    pub fn build(self) -> AcpiTable {
        AcpiTable {}
    }
}
