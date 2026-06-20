use vm_pci::types::configuration_space::header::PCI_COMMAND_INTX_DISABLE;
use vm_pci::types::configuration_space::header::PCI_STATUS_INTERRUPT;
use vm_pci::types::configuration_space::header::type0::Type0Header;

use crate::transport::common::control_register::ControlRegister;
use crate::transport::pci::VirtioPciDevice;
use crate::transport::pci::VirtioPciTransport;

impl<D> VirtioPciTransport<D>
where
    D: VirtioPciDevice,
{
    pub fn read_isr(&self, _offset: u64, data: &mut [u8]) {
        let mut common = self.common.lock().unwrap();

        let isr = common.read_reg(ControlRegister::InterruptStatus).unwrap();
        data[0] = isr as u8;

        /*
         * From `4.1.4.5.1 Device Requirements: ISR status capability`
         * - The device MUST reset ISR status to 0 on driver read.
         */
        common
            .write_reg(ControlRegister::InterruptStatus, 0)
            .unwrap();

        let mut cfg = self
            .interrupt_dispatcher
            .configuration_space
            .lock()
            .unwrap();
        let header = cfg.as_header_mut::<Type0Header>();
        if header.common.command & PCI_COMMAND_INTX_DISABLE == 0 {
            header.common.status &= !PCI_STATUS_INTERRUPT;
            let irq = self.interrupt_dispatcher.legacy_int.as_ref().unwrap();
            self.interrupt_dispatcher
                .irq_chip
                .trigger_irq(*irq as u32, false);
        }
    }

    pub fn write_isr(&self, _offset: u64, _data: &[u8]) {
        unreachable!()
    }
}
