#[cfg(target_os = "macos")]
pub mod hvp;

#[cfg(target_os = "linux")]
pub mod kvm;

pub mod hypervisor;
pub mod vcpu;
pub mod vm;
