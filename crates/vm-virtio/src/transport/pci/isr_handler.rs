use std::sync::Arc;
use std::sync::Mutex;

use vm_mm::allocator::MemoryContainer;
use vm_pci::device::function::BarHandler;

use crate::transport::VirtioDev;
use crate::transport::control_register::ControlRegister;
use crate::transport::pci::VirtioPciDevice;

pub struct IsrHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    pub dev: Arc<Mutex<VirtioDev<C, D>>>,
}

impl<C, D> BarHandler for IsrHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    fn read(&self, _offset: u64, data: &mut [u8]) {
        let mut dev = self.dev.lock().unwrap();

        let isr = dev.read_reg(ControlRegister::InterruptStatus);
        data[0] = isr as u8;

        /*
         * From `4.1.4.5.1 Device Requirements: ISR status capability`
         * - The device MUST reset ISR status to 0 on driver read.
         */
        dev.write_reg(ControlRegister::InterruptStatus, 0).unwrap();
        dev.device.trigger_irq(false);
    }

    fn write(&self, _offset: u64, _data: &[u8]) {
        unreachable!()
    }
}
