use vm_core::device::Device;
use vm_core::device::mmio::MmioRange;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_fdt::FdtWriter;

use crate::pci::root_complex::PciRootComplex;

#[derive(Debug)]
struct DeviceSel {
    bus: u8,
    device: u8,
    func: u8,
    offset: u16,
}

impl From<u64> for DeviceSel {
    fn from(addr: u64) -> DeviceSel {
        DeviceSel {
            bus: (addr >> 20) as u8,
            device: ((addr >> 15) & 0x1f) as u8,
            func: ((addr >> 12) & 0x7) as u8,
            offset: (addr & 0xfff) as u16,
        }
    }
}

pub struct PciRootComplexMmio {
    mmio_range: MmioRange,
    internal: PciRootComplex,
}

impl PciRootComplexMmio {
    pub fn new(mmio_range: MmioRange) -> Self {
        PciRootComplexMmio {
            mmio_range,
            internal: Default::default(),
        }
    }
}

impl Device for PciRootComplexMmio {
    fn name(&self) -> String {
        "pci-root-complex".to_string()
    }
}

impl MmioDevice for PciRootComplexMmio {
    fn mmio_range(&self) -> MmioRange {
        self.mmio_range
    }

    fn mmio_read(&mut self, offset: u64, _len: usize, data: &mut [u8]) {
        let sel = DeviceSel::from(offset);

        self.internal
            .handle_ecam_read(sel.bus, sel.device, sel.func, sel.offset, data);
    }

    fn mmio_write(&mut self, offset: u64, _len: usize, data: &[u8]) {
        let sel = DeviceSel::from(offset);

        self.internal
            .handle_ecam_write(sel.bus, sel.device, sel.func, sel.offset, data);
    }

    fn generate_dt(&self, fdt: &mut FdtWriter) -> Result<(), vm_fdt::Error> {
        let node = fdt.begin_node(&format!("pci@{:x}", self.mmio_range.start))?;
        fdt.property_string("compatible", "pci-host-ecam-generic")?;
        fdt.property_string("device-type", "pci")?;
        fdt.property_u32("#size-cells", 2)?;
        fdt.property_u32("#address-cells", 3)?;
        // interrupt
        fdt.property_array_u64("reg", &[self.mmio_range.start, self.mmio_range.len as u64])?;
        fdt.end_node(node)?;

        Ok(())
    }
}
