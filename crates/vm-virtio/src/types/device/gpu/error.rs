use thiserror::Error;

#[derive(Error, Debug)]
pub enum VirtioGpuError {
    #[error("unknwon ctrl type {0}")]
    UnknownCtrlType(u32),
}
