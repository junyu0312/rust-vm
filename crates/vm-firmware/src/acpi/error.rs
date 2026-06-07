use thiserror::Error;

#[derive(Error, Debug)]
pub enum AcpiError {
    #[error("Field {0} not set")]
    FieldNotSet(&'static str),

    #[error("Failed to copy ACPI table to guest memory")]
    CopyToMemory(#[from] vm_mm::error::Error),
}
