#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
pub mod bzimage;

#[cfg(target_arch = "aarch64")]
pub mod image;
