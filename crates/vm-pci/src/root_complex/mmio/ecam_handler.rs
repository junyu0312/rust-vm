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
