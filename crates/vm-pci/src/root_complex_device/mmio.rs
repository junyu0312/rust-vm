use std::ops::Range;
use std::sync::Arc;
use std::sync::Mutex;

use vm_core::device::error::DeviceError;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_core::utils::address_space::AddressSpace;
use vm_fdt::FdtWriter;
use vm_utils::range_allocator::RangeAllocator;

use crate::root_complex::pci_root_complex::PciRootComplex;

pub type PciToGpaMapping = AddressSpace<u64, u64>;

mod arch;
// mod bar_handler;
mod ecam_handler;

pub struct MmioTransport {
    pub(crate) ecam_range: Range<u64>,
    // pci_to_gpa_mapping: PciToGpaMapping,
    pub(crate) pci_bar_mmio_window: Range<u64>,
    internal: Arc<Mutex<PciRootComplex>>,
}

impl MmioTransport {
    pub fn new(
        mmio_allocator: &mut RangeAllocator<u64>,
        ecam_range: Range<u64>,
        pci_bar_mmio_window: Range<u64>,
        // physica_start: u64,
        // pci_address_space_len: usize,
        internal: Arc<Mutex<PciRootComplex>>,
    ) -> Result<Self, DeviceError> {
        let _ = mmio_allocator
            .reserve(
                ecam_range.start,
                (ecam_range.end - ecam_range.start) as usize,
            )
            .map_err(|_| DeviceError::AllocResource)?;
        let _ = mmio_allocator
            .reserve(
                pci_bar_mmio_window.start,
                (pci_bar_mmio_window.end - pci_bar_mmio_window.start) as usize,
            )
            .map_err(|_| DeviceError::AllocResource)?;

        // let mut pci_to_gpa_mapping = PciToGpaMapping::default();
        // pci_to_gpa_mapping
        //     .try_insert(0..(pci_address_space_len as u64), physica_start)
        //     .unwrap();

        Ok(MmioTransport {
            ecam_range,
            pci_bar_mmio_window,
            internal,
        })
    }
}

impl MmioDevice for MmioTransport {
    fn mmio_ranges(&self) -> Vec<Range<u64>> {
        vec![self.ecam_range.clone(), self.pci_bar_mmio_window.clone()]
    }
    /*
    fn mmio_range_handlers(&self) -> Vec<Box<dyn MmioHandler>> {
        /*
        let mut handlers =
            Vec::<Box<dyn MmioHandler>>::with_capacity(self.pci_to_gpa_mapping.len() + 1);

        handlers.push(Box::new(EcamHandler::new(
            self.ecam_range.clone(),
            self.internal.clone(),
        )));
        for (&pci_address, &(len, gpa)) in self.pci_to_gpa_mapping.iter() {
            handlers.push(Box::new(DeviceMmioHandler::new(
                gpa..gpa + len as u64,
                pci_address..pci_address + len as u64,
                self.internal.clone(),
            )));
        }

        handlers
        */
        todo!()
    }
    */

    fn mmio_read(&self, addr: u64, buf: &mut [u8]) -> Result<(), DeviceError> {
        if self.ecam_range.contains(&addr) {
            let offset = addr - self.ecam_range.start;
            self.handle_ecam_read(offset, buf);

            return Ok(());
        }

        if self.pci_bar_mmio_window.contains(&addr) {
            todo!()
        }

        unreachable!()
    }

    fn mmio_write(&self, addr: u64, buf: &[u8]) -> Result<(), DeviceError> {
        if self.ecam_range.contains(&addr) {
            let offset = addr - self.ecam_range.start;
            self.handle_ecam_write(offset, buf);

            return Ok(());
        }

        if self.pci_bar_mmio_window.contains(&addr) {
            todo!()
        }

        unreachable!()
    }

    fn generate_dt(&self, fdt: &mut FdtWriter) -> Result<(), DeviceError> {
        let node = fdt.begin_node(&format!("pcie@{:x}", self.ecam_range.start))?;
        fdt.property_string("compatible", "pci-host-ecam-generic")?;
        fdt.property_string("device_type", "pci")?;
        fdt.property_u32("#size-cells", 2)?;
        fdt.property_u32("#address-cells", 3)?;
        fdt.property_u32("#interrupt-cells", 1)?;

        {
            todo!()
            /*
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
            */
        }
        fdt.property_array_u32("bus-range", &[0, 0])?;
        fdt.property_array_u64(
            "reg",
            &[
                self.ecam_range.start,
                self.ecam_range.end - self.ecam_range.start,
            ],
        )?;

        self.generate_device_tree_arch(fdt)?;

        fdt.end_node(node)?;

        Ok(())
    }
}
