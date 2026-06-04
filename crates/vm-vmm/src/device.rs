use std::sync::Arc;

use vm_core::arch::irq::InterruptController;
use vm_device::device::Device;
use vm_mm::manager::MemoryAddressSpace;

use crate::device::error::InitDeviceError;
use crate::service::monitor::builder::MonitorServerBuilder;

pub(crate) mod error;

mod arch;
mod irq_allocation;

pub trait InitDevice {
    fn init_devices(
        &mut self,
        monitor_server_builder: &mut MonitorServerBuilder,
        mm: Arc<MemoryAddressSpace>,
        devices: &[Device],
        irq_chip: Arc<dyn InterruptController>,
    ) -> Result<(), InitDeviceError>;
}
