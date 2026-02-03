pub mod allocator;
pub mod manager;
pub mod region;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to allocate anonymous memory, len: {len}")]
    AllocAnonymousMemoryFailed { len: usize },

    #[error("try to access an uninitialized memory")]
    MemoryIsNotAllocated,

    #[error("memory already allocated, cannot allocate again")]
    MemoryAlreadyAllocated,

    #[error("try to access invalid gpa: {0}")]
    AccessInvalidGpa(u64),

    #[error("access memory overflow")]
    MemoryOverflow,
}
