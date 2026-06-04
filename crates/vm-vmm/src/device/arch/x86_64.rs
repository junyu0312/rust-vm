use std::hint::black_box;
use std::sync::Arc;

use vm_core::arch::irq::InterruptController;
use vm_core::device_manager::DeviceManager;
use vm_device::device::Device;
use vm_device::device::cmos::Cmos;
use vm_device::device::dummy::Dummy;
use vm_device::device::i8042::I8042;
use vm_device::device::post_debug::PostDebug;
use vm_device::device::uart8250::Uart8250;
use vm_mm::manager::MemoryAddressSpace;
use vm_pci::root_complex::pio::PciRootComplexPio;

use crate::device::InitDevice;
use crate::device::error::InitDeviceError;
use crate::device::irq_allocation::IrqAllocation;
use crate::service::monitor::builder::MonitorServerBuilder;

impl InitDevice for DeviceManager {
    fn init_devices(
        &mut self,
        _monitor_server_builder: &mut MonitorServerBuilder,
        _mm: Arc<MemoryAddressSpace>,
        devices: &[Device],
        irq_chip: Arc<dyn InterruptController>,
    ) -> Result<(), InitDeviceError> {
        let mut irq_allocation = IrqAllocation::new(0);
        black_box(irq_allocation.alloc());

        let uart8250_com1 = Uart8250::<4>::new(Some(0x3f8), irq_chip.clone());
        let uart8250_com2 = Uart8250::<3>::new(Some(0x2f8), irq_chip.clone());
        let uart8250_com3 = Uart8250::<4>::new(Some(0x3e8), irq_chip.clone());
        let uart8250_com4 = Uart8250::<3>::new(Some(0x2e8), irq_chip.clone());
        let cmos = Cmos;
        let post_debug = PostDebug;
        let dummy = Dummy;
        let i8042 = I8042::new(irq_chip);
        let pci_rc = PciRootComplexPio::default();

        self.register_pio_device(Box::new(uart8250_com1))?;
        self.register_pio_device(Box::new(uart8250_com2))?;
        self.register_pio_device(Box::new(uart8250_com3))?;
        self.register_pio_device(Box::new(uart8250_com4))?;
        self.register_pio_device(Box::new(cmos))?;
        self.register_pio_device(Box::new(post_debug))?;
        self.register_pio_device(Box::new(dummy))?;
        self.register_pio_device(Box::new(i8042))?;
        self.register_pio_device(Box::new(pci_rc))?;

        for device in devices {
            match device {
                Device::GicV3 => todo!(),
                Device::VirtioMmioBalloon => {
                    todo!()
                }
                Device::VirtioMmioEntropy => todo!(),
                Device::VirtioPciEntropy => todo!(),
                Device::VfioPci { .. } => todo!(),
            }
        }

        Ok(())
    }
}
