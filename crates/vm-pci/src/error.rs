#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no space to allocate capability")]
    CapNoSpace,
    #[error("the capability is too large")]
    CapTooLarge,
}
