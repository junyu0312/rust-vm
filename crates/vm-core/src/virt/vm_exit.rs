#[derive(Debug, thiserror::Error)]
pub enum Error {}

#[derive(Debug)]
pub enum VmExitReason {
    Unknown,
    MMIO {
        gpa: u64,
        data: Option<u64>,
        len: u32,
        is_write: bool,
        is_32bit_inst: bool,
    },
}

pub enum HandleVmExitResult {
    Continue,
    NextInst,
}
