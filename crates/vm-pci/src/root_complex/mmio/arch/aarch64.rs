use vm_core::irq::Phandle;
use vm_core::irq::arch::aarch64::GIC_SPI;
use vm_core::irq::arch::aarch64::IRQ_TYPE_LEVEL_HIGH;
use vm_fdt::FdtWriter;

use crate::device::interrupt::legacy::InterruptPin;
use crate::root_complex::mmio::PciRootComplexMmio;

struct InterruptMapEntry {
    pci_addr_high: u32,
    pci_addr_mid: u32,
    pci_addr_low: u32,
    pin: u32,
    gic_phandle: u32,
    gic_addr_high: u32,
    gic_addr_low: u32,
    gic_irq_type: u32,
    gic_irq_num: u32,
    gic_irq_flags: u32,
}

impl InterruptMapEntry {
    fn into_array(self) -> [u32; 10] {
        [
            self.pci_addr_high,
            self.pci_addr_mid,
            self.pci_addr_low,
            self.pin,
            self.gic_phandle,
            self.gic_addr_high,
            self.gic_addr_low,
            self.gic_irq_type,
            self.gic_irq_num,
            self.gic_irq_flags,
        ]
    }
}

impl PciRootComplexMmio {
    pub fn generate_device_tree_arch(&self, fdt: &mut FdtWriter) -> Result<(), vm_fdt::Error> {
        fdt.property_array_u32("interrupt-map-mask", &[0, 0, 0, 7])?;
        // TODO: hard code, virtio-pci-blk
        let entry = InterruptMapEntry {
            pci_addr_high: 0x800,
            pci_addr_mid: 0,
            pci_addr_low: 0,
            pin: InterruptPin::INTA as u32,
            gic_phandle: Phandle::GIC as u32,
            gic_addr_high: 0,
            gic_addr_low: 0,
            gic_irq_type: GIC_SPI,
            gic_irq_num: 10,
            gic_irq_flags: IRQ_TYPE_LEVEL_HIGH,
        };
        fdt.property_array_u32("interrupt-map", &entry.into_array())?;
        fdt.property_u32("msi-parent", Phandle::MSI as u32)?;

        Ok(())
    }
}
