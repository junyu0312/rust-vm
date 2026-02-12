use std::sync::Arc;
use std::sync::Mutex;

use vm_core::device::Device;
use vm_core::device::mmio::MmioRange;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_core::device::mmio::mmio_device::MmioHandler;
use vm_fdt::FdtWriter;

use crate::device::pci_device::PciDevice;
use crate::root_complex::PciRootComplex;
use crate::root_complex::mmio::device_mmio_handler::DeviceMmioHandler;
use crate::root_complex::mmio::ecam_handler::EcamHandler;

#[cfg(target_arch = "x86_64")]
fn todo_irq() {
    todo!()
}

#[cfg(target_arch = "aarch64")]
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

#[cfg(target_arch = "aarch64")]
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
    ecam_range: MmioRange,
    bar_mmio_ranges: Vec<MmioRange>,
    physical_address_start: u64,
    pci_address_space_start: u64,
    pci_address_space_len: u64,
    internal: Arc<Mutex<PciRootComplex>>,
}

impl PciRootComplexMmio {
    pub fn new(
        ecam_range: MmioRange,
        physical_address_start: u64,
        pci_address_space_len: u64,
    ) -> Self {
        PciRootComplexMmio {
            ecam_range,
            bar_mmio_ranges: vec![MmioRange {
                start: physical_address_start,
                len: pci_address_space_len.try_into().unwrap(),
            }],
            physical_address_start,
            pci_address_space_start: 0,
            pci_address_space_len,
            internal: Default::default(),
        }
    }

    pub fn register_device(&self, device: PciDevice) -> Result<(), PciDevice> {
        let mut rc = self.internal.lock().unwrap();
        rc.register_device(device)
    }
}

impl Device for PciRootComplexMmio {
    fn name(&self) -> String {
        "pci-root-complex".to_string()
    }
}

mod ecam_handler {
    use std::sync::Arc;
    use std::sync::Mutex;

    use vm_core::device::mmio::MmioRange;
    use vm_core::device::mmio::mmio_device::MmioHandler;

    use crate::root_complex::PciRootComplex;
    use crate::root_complex::mmio::DeviceSel;

    pub struct EcamHandler {
        mmio_range: MmioRange,
        rc: Arc<Mutex<PciRootComplex>>,
    }

    impl EcamHandler {
        pub fn new(mmio_range: MmioRange, rc: Arc<Mutex<PciRootComplex>>) -> Self {
            EcamHandler { mmio_range, rc }
        }
    }

    impl MmioHandler for EcamHandler {
        fn mmio_range(&self) -> MmioRange {
            self.mmio_range
        }

        fn mmio_read(&self, offset: u64, _len: usize, data: &mut [u8]) {
            let sel = DeviceSel::from(offset);

            let rc = self.rc.lock().unwrap();

            rc.handle_ecam_read(sel.bus, sel.device, sel.func, sel.offset, data);
        }

        fn mmio_write(&self, offset: u64, _len: usize, data: &[u8]) {
            let sel = DeviceSel::from(offset);

            let mut rc = self.rc.lock().unwrap();

            rc.handle_ecam_write(sel.bus, sel.device, sel.func, sel.offset, data);
        }
    }
}

mod device_mmio_handler {
    use std::sync::Arc;
    use std::sync::Mutex;

    use vm_core::device::mmio::MmioRange;
    use vm_core::device::mmio::mmio_device::MmioHandler;

    use crate::root_complex::PciRootComplex;

    pub struct DeviceMmioHandler {
        bar_mmio_range: MmioRange,
        rc: Arc<Mutex<PciRootComplex>>,
    }

    impl DeviceMmioHandler {
        pub fn new(bar_mmio_range: MmioRange, rc: Arc<Mutex<PciRootComplex>>) -> Self {
            DeviceMmioHandler { bar_mmio_range, rc }
        }
    }

    impl MmioHandler for DeviceMmioHandler {
        fn mmio_range(&self) -> MmioRange {
            self.bar_mmio_range
        }

        fn mmio_read(&self, offset: u64, len: usize, data: &mut [u8]) {
            assert_eq!(len, data.len());
            let rc = self.rc.lock().unwrap();
            // TODO: It's incorrect, it's working because we only have one pci-physical address mapping
            let handler = rc.mmio_router.get_handler(offset);
            if let Some(handler) = handler {
                handler.read(offset, data);
            }
        }

        fn mmio_write(&self, offset: u64, len: usize, data: &[u8]) {
            assert_eq!(len, data.len());
            let rc = self.rc.lock().unwrap();
            // TODO: It's incorrect, it's working because we only have one pci-physical address mapping
            let handler = rc.mmio_router.get_handler(offset);
            if let Some(handler) = handler {
                handler.write(offset, data);
            }
        }
    }
}

impl MmioDevice for PciRootComplexMmio {
    fn mmio_range_handlers(&self) -> Vec<Box<dyn MmioHandler>> {
        let mut handlers =
            Vec::<Box<dyn MmioHandler>>::with_capacity(self.bar_mmio_ranges.len() + 1);

        handlers.push(Box::new(EcamHandler::new(
            self.ecam_range,
            self.internal.clone(),
        )));
        for bar_mmio_range in &self.bar_mmio_ranges {
            handlers.push(Box::new(DeviceMmioHandler::new(
                *bar_mmio_range,
                self.internal.clone(),
            )));
        }

        handlers
    }

    fn generate_dt(&self, fdt: &mut FdtWriter) -> Result<(), vm_fdt::Error> {
        let node = fdt.begin_node(&format!("pcie@{:x}", self.ecam_range.start))?;
        fdt.property_string("compatible", "pci-host-ecam-generic")?;
        fdt.property_string("device_type", "pci")?;
        fdt.property_u32("#size-cells", 2)?;
        fdt.property_u32("#address-cells", 3)?;
        fdt.property_u32("#interrupt-cells", 1)?;
        fdt.property_array_u32(
            "ranges",
            &[
                0x0200_0000,
                (self.pci_address_space_start >> 32) as u32,
                (self.pci_address_space_start) as u32,
                (self.physical_address_start >> 32) as u32,
                (self.physical_address_start) as u32,
                (self.pci_address_space_len >> 32) as u32,
                (self.pci_address_space_len) as u32,
            ],
        )?;
        fdt.property_array_u32("bus-range", &[0, 0])?;
        fdt.property_array_u64("reg", &[self.ecam_range.start, self.ecam_range.len as u64])?;

        #[cfg(target_arch = "aarch64")]
        {
            use vm_core::irq::Phandle;
            use vm_core::irq::arch::aarch64::GIC_SPI;
            use vm_core::irq::arch::aarch64::IRQ_TYPE_LEVEL_HIGH;

            use crate::device::interrupt::legacy::InterruptPin;

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
        }

        #[cfg(target_arch = "x86_64")]
        {
            todo_irq();
        }

        fdt.end_node(node)?;

        Ok(())
    }
}
