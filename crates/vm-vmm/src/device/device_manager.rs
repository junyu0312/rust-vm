use std::path::Path;
use std::slice::Iter;
use std::slice::IterMut;
use std::sync::Arc;

use vm_core::arch::irq::InterruptController;
use vm_core::device::mmio::layout::MmioLayout;
use vm_core::device::mmio::mmio_as_manager::MmioAddressSpaceManager;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_core::device::pio::pio_device::PioDevice;
use vm_core::utils::address_space::AddressSpaceError;
use vm_device::device::Device;
use vm_mm::manager::MemoryAddressSpace;
use vm_pci::root_complex::PciRootComplexOps;
use vm_vfio::vfio::VfioContainer;
use vm_vfio::vfio::VfioContainerOps;
use vm_vfio::vfio_pci::device::VfioPciDevice;

use crate::device::device_manager::pio::PioAddressSpaceManager;
use crate::device::error::InitDeviceError;
use crate::service::monitor::builder::MonitorServerBuilder;

pub(crate) mod snapshot;

mod arch;
mod irq_allocation;
mod mmio;
mod pio;

pub struct DeviceManager {
    pub pio_manager: PioAddressSpaceManager,
    pub mmio_manager: MmioAddressSpaceManager,
}

impl DeviceManager {
    pub fn new(mmio_layout: MmioLayout) -> Self {
        DeviceManager {
            pio_manager: PioAddressSpaceManager::default(),
            mmio_manager: MmioAddressSpaceManager::new(mmio_layout),
        }
    }

    pub fn register_pio_device(
        &mut self,
        device: Box<dyn PioDevice>,
    ) -> Result<(), AddressSpaceError> {
        self.pio_manager.register(device)
    }

    pub fn register_mmio_device(
        &mut self,
        device: Box<dyn MmioDevice>,
    ) -> Result<(), AddressSpaceError> {
        self.mmio_manager.register(device)
    }

    pub fn mmio_devices(&self) -> Iter<'_, Box<dyn MmioDevice>> {
        self.mmio_manager.devices()
    }

    pub fn mmio_devices_mut(&mut self) -> IterMut<'_, Box<dyn MmioDevice>> {
        self.mmio_manager.devices_mut()
    }

    fn init_vfio_container(&mut self) -> Result<VfioContainer, InitDeviceError> {
        Ok(VfioContainer::new()?)
    }

    fn init_vfio_device(
        &self,
        name: String,
        container: &dyn VfioContainerOps,
        path: &Path,
    ) -> Result<VfioPciDevice, InitDeviceError> {
        let vfio_device = container.new_device(path)?;

        let vfio_pci_device = VfioPciDevice::new(name, vfio_device)?;

        Ok(vfio_pci_device)
    }

    fn init_device(
        &self,
        vfio_container: &dyn VfioContainerOps,
        pci_root_complex: &dyn PciRootComplexOps,
        device: &Device,
    ) -> Result<(), InitDeviceError> {
        match device {
            Device::GicV3 => todo!(),
            Device::VirtioMmioBalloon => todo!(),
            Device::VirtioMmioEntropy => todo!(),
            Device::VirtioPciEntropy => todo!(),
            #[cfg(target_os = "linux")]
            Device::VfioPci { name, path } => {
                let dev = self.init_vfio_device(name.to_string(), vfio_container, path)?;
                pci_root_complex
                    .register_device(Box::new(dev))
                    .map_err(|_| vm_pci::error::Error::FailedRegisterPciDevice)?;
            }
        }

        Ok(())
    }

    pub fn init(
        &mut self,
        monitor_server_builder: &mut MonitorServerBuilder,
        mm: Arc<MemoryAddressSpace>,
        devices: &[Device],
        irq_chip: Arc<dyn InterruptController>,
    ) -> Result<(), InitDeviceError> {
        let vfio_container = self.init_vfio_container()?;

        self.init_arch(
            &vfio_container,
            monitor_server_builder,
            mm,
            devices,
            irq_chip,
        )?;

        Ok(())
    }
}
