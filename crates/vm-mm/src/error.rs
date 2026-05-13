use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to allocate anonymous memory, len: {len}")]
    AllocAnonymousMemoryFailed { len: usize },

    #[error("try to access invalid gpa: {0}")]
    AccessInvalidGpa(u64),

    #[error("access memory overflow")]
    MemoryOverflow,

    #[error("failed to save memory snapshot, error: {0}")]
    Save(Box<dyn std::error::Error + Send + Sync>),
}
