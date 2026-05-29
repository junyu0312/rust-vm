use std::io::Read;
use std::io::Write;

use vm_core::arch::irq::InterruptController;
use vm_core::arch::irq::Phandle;
use vm_core::arch::irq::error::IrqChipError;

#[derive(Default)]
pub struct GicV3 {}

impl InterruptController for GicV3 {
    fn trigger_irq(&self, _irq_line: u32, _active: bool) {
        todo!()
    }

    fn send_msi(&self, _intid: u32) {
        todo!()
    }

    fn write_device_tree(&self, _fdt: &mut vm_fdt::FdtWriter) -> Result<Phandle, IrqChipError> {
        todo!()
    }

    fn save(&self, _write: &mut dyn Write) -> Result<(), IrqChipError> {
        todo!()
    }

    fn load(&mut self, _read: &mut dyn Read) -> Result<(), IrqChipError> {
        todo!()
    }
}
