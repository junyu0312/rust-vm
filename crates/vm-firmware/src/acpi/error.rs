use thiserror::Error;

#[derive(Error, Debug)]
pub enum AcpiError {
    #[error("Failed to copy ACPI table to guest memory")]
    CopyToMemory,
}
