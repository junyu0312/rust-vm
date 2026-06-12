use crate::transport::control_register::ControlRegister;
use crate::transport::pci::VirtioPciDevice;
use crate::transport::pci::VirtioPciTransport;

impl<D> VirtioPciTransport<D>
where
    D: VirtioPciDevice,
{
    pub fn read_isr(&self, _offset: u64, data: &mut [u8]) {
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

    pub fn write_isr(&self, _offset: u64, _data: &[u8]) {
        unreachable!()
    }
}
