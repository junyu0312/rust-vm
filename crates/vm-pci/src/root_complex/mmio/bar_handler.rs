use std::sync::Arc;
use std::sync::Mutex;

use vm_core::device::mmio::MmioRange;
use vm_core::device::mmio::mmio_device::MmioHandler;

use crate::root_complex::PciRootComplex;

pub struct DeviceMmioHandler {
    gpa_mmio_range: MmioRange,
    pci_mmio_range: MmioRange,
    rc: Arc<Mutex<PciRootComplex>>,
}

impl DeviceMmioHandler {
    pub fn new(
        gpa_mmio_range: MmioRange,
        pci_mmio_range: MmioRange,
        rc: Arc<Mutex<PciRootComplex>>,
    ) -> Self {
        DeviceMmioHandler {
            gpa_mmio_range,
            pci_mmio_range,
            rc,
        }
    }
}

impl MmioHandler for DeviceMmioHandler {
    fn mmio_range(&self) -> MmioRange {
        self.gpa_mmio_range // Must be gpa_mmio_range for dispatch vm_exit
    }

    // offset is mmio pa - self.gpa_mmio_range.start already
    fn mmio_read(&self, offset: u64, len: usize, data: &mut [u8]) {
        assert_eq!(len, data.len());
        let rc = self.rc.lock().unwrap();
        let pci_address_offset = offset + self.pci_mmio_range.start;
        let handler = rc.mmio_router.get_handler(pci_address_offset);
        if let Some((mmio_range, handler)) = handler {
            handler.read(offset - mmio_range.start, data);
        }
    }

    // offset is mmio pa - self.gpa_mmio_range.start already
    fn mmio_write(&self, offset: u64, len: usize, data: &[u8]) {
        assert_eq!(len, data.len());
        let rc = self.rc.lock().unwrap();
        let pci_address_offset = offset + self.pci_mmio_range.start;
        let handler = rc.mmio_router.get_handler(pci_address_offset);
        if let Some((mmio_range, handler)) = handler {
            handler.write(offset - mmio_range.start, data);
        }
    }
}
