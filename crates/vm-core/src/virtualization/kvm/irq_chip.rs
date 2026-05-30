use std::io::Read;
use std::io::Write;

use vm_fdt::FdtWriter;

use crate::arch::irq::InterruptController;
use crate::arch::irq::Phandle;
use crate::arch::irq::error::IrqChipError;

pub struct KvmIrqChip {}

impl InterruptController for KvmIrqChip {
    fn trigger_irq(&self, _irq_line: u32, _active: bool) {
        todo!()
    }

    fn send_msi(&self, _intid: u32) {
        todo!()
    }

    fn write_device_tree(&self, _fdt: &mut FdtWriter) -> Result<Phandle, IrqChipError> {
        todo!()
    }

    fn save(&self, _write: &mut dyn Write) -> Result<(), IrqChipError> {
        todo!()
    }

    fn load(&mut self, _read: &mut dyn Read) -> Result<(), IrqChipError> {
        todo!()
    }
}
