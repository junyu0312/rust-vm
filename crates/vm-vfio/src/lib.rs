// #![deny(warnings)]

#[cfg(target_os = "linux")]
pub mod error;
#[cfg(target_os = "linux")]
pub mod vfio;
#[cfg(target_os = "linux")]
pub mod vfio_pci;
