#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no space to allocate capability")]
    CapNoSpace,
    #[error("the cap_id is invalid")]
    InvalidCapId,
    #[error("the cap_version is invalid")]
    InvalidCapVersion,
}
