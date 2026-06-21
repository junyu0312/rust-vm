use std::io::Read;
use std::io::Write;
use std::sync::Arc;

use kvm_bindings::kvm_msi;
use kvm_ioctls::VmFd;
use vm_fdt::FdtWriter;

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::irq::GIC_SPI_START;
use crate::arch::irq::InterruptController;
use crate::arch::irq::Phandle;
use crate::arch::irq::error::IrqChipError;

pub struct KvmIrqChip {
    pub vm_fd: Arc<VmFd>,
}

impl InterruptController for KvmIrqChip {
    fn trigger_irq(&self, irq: u32, active: bool) {
        #[cfg(target_arch = "x86_64")]
        let _ = self.vm_fd.set_irq_line(irq, active);

        #[cfg(target_arch = "aarch64")]
        let _ = self.vm_fd.set_irq_line(irq + GIC_SPI_START, active);
    }

    fn send_msi(&self, address_lo: u32, address_hi: u32, data: u32) {
        let msi = kvm_msi {
            address_lo,
            address_hi,
            data,
            ..Default::default()
        };

        self.vm_fd.signal_msi(msi).unwrap();
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
