use std::sync::Arc;
use std::sync::Mutex;

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::firmware::psci::Psci;
use crate::arch::irq::InterruptController;
use crate::device_manager::vm_exit::DeviceVmExitHandler;
use crate::error::Error;
use crate::vcpu::vcpu::Vcpu;

#[cfg(feature = "kvm")]
pub mod kvm;

#[cfg(feature = "hvp")]
pub mod hvp;

pub enum SetUserMemoryRegionFlags {
    ReadWriteExec,
}

pub trait Vm {
    fn create_vcpu(
        &self,
        vcpu_id: usize,
        device_vm_exit_handler: Arc<dyn DeviceVmExitHandler>,
        #[cfg(target_arch = "aarch64")] psci: Arc<dyn Psci>,
    ) -> Result<Arc<Mutex<dyn Vcpu>>, Error>;

    fn create_irq_chip(&self) -> Result<Arc<dyn InterruptController>, Error>;

    fn set_user_memory_region(
        &self,
        userspace_addr: u64,
        guest_phys_addr: u64,
        memory_size: usize,
        flags: SetUserMemoryRegionFlags,
    ) -> Result<(), Error>;
}

pub trait Virt {
    fn create_vm(&self) -> Result<Arc<dyn Vm>, Error>;
}
