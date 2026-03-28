use std::sync::Arc;
use std::sync::Mutex;

use vm_pci::device::function::BarHandler;

use crate::transport::VirtioDev;
use crate::transport::control_register::ControlRegister;
use crate::transport::pci::VirtioPciDevice;

pub struct IsrHandler<D>
where
    D: VirtioPciDevice,
{
    pub dev: Arc<Mutex<VirtioDev<D>>>,
}

impl<D> BarHandler for IsrHandler<D>
where
    D: VirtioPciDevice,
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
