use std::sync::Arc;
use std::sync::Mutex;

use vm_core::device::Device;
use vm_core::device::address_space::AddressSpace;
use vm_core::device::mmio::MmioRange;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_core::device::mmio::mmio_device::MmioHandler;
use vm_fdt::FdtWriter;

use crate::device::pci_device::PciDevice;
use crate::root_complex::PciRootComplex;
use crate::root_complex::mmio::bar_handler::DeviceMmioHandler;
use crate::root_complex::mmio::ecam_handler::EcamHandler;

mod arch;
mod bar_handler;
mod ecam_handler;

type PciToGpaMapping = AddressSpace<u64, u64>;

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
    pci_to_gpa_mapping: PciToGpaMapping,
    internal: Arc<Mutex<PciRootComplex>>,
}

impl PciRootComplexMmio {
    pub fn new(
        ecam_range: MmioRange,
        physical_address_start: u64,
        pci_address_space_len: usize,
    ) -> Self {
        let mut pci_to_gpa_mapping = PciToGpaMapping::new();
        pci_to_gpa_mapping
            .try_insert(
                MmioRange {
                    start: 0,
                    len: pci_address_space_len,
                },
                physical_address_start,
            )
            .unwrap();

        PciRootComplexMmio {
            ecam_range,
            pci_to_gpa_mapping,
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

impl MmioDevice for PciRootComplexMmio {
    fn mmio_range_handlers(&self) -> Vec<Box<dyn MmioHandler>> {
        let mut handlers =
            Vec::<Box<dyn MmioHandler>>::with_capacity(self.pci_to_gpa_mapping.len() + 1);

        handlers.push(Box::new(EcamHandler::new(
            self.ecam_range,
            self.internal.clone(),
        )));
        for (&pci_address, &(len, gpa)) in self.pci_to_gpa_mapping.iter() {
            handlers.push(Box::new(DeviceMmioHandler::new(
                MmioRange { start: gpa, len },
                MmioRange {
                    start: pci_address,
                    len,
                },
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

        {
            let mut ranges_vec: Vec<u32> = Vec::new();
            self.pci_to_gpa_mapping
                .iter()
                .for_each(|(&pci_addr, &(len, gpa))| {
                    ranges_vec.extend_from_slice(&[
                        0x0200_0000, // MEM
                        (pci_addr >> 32) as u32,
                        pci_addr as u32,
                        (gpa >> 32) as u32,
                        gpa as u32,
                        (len >> 32) as u32,
                        len as u32,
                    ]);
                });
            fdt.property_array_u32("ranges", &ranges_vec[..])?;
        }
        fdt.property_array_u32("bus-range", &[0, 0])?;
        fdt.property_array_u64("reg", &[self.ecam_range.start, self.ecam_range.len as u64])?;

        self.generate_device_tree_arch(fdt)?;

        fdt.end_node(node)?;

        Ok(())
    }
}
