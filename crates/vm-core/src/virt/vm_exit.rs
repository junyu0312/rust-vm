#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to handle mmio, err: {0}")]
    MmioErr(String),
}

#[derive(Debug)]
pub enum VmExitReason {
    Unknown,
    MMIO {
        gpa: u64,
        data: Option<u64>,
        len: usize,
        is_write: bool,
    },
}

pub enum HandleVmExitResult {
    Continue,
    AdvancePc,
}
