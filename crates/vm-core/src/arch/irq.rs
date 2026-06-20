use std::io::Read;
use std::io::Write;

use vm_fdt::FdtWriter;

use crate::arch::irq::error::IrqChipError;

pub mod error;

#[repr(u32)]
pub enum Phandle {
    GIC = 0x1,
    MSI = 0x2,
}

pub trait InterruptController: Send + Sync + 'static {
    fn trigger_irq(&self, irq_line: u32, active: bool);

    fn send_msi(&self, address_lo: u32, address_hi: u32, data: u32);

    fn write_device_tree(&self, fdt: &mut FdtWriter) -> Result<Phandle, IrqChipError>;

    fn save(&self, write: &mut dyn Write) -> Result<(), IrqChipError>;

    fn load(&mut self, read: &mut dyn Read) -> Result<(), IrqChipError>;
}
