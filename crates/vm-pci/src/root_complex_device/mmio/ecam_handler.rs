use crate::root_complex_device::mmio::MmioTransport;

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

impl MmioTransport {
    pub fn handle_ecam_read(&self, offset: u64, data: &mut [u8]) {
        let sel = DeviceSel::from(offset);

        self.internal
            .lock()
            .unwrap()
            .handle_ecam_read(sel.bus, sel.device, sel.func, sel.offset, data);
    }

    pub fn handle_ecam_write(&self, offset: u64, data: &[u8]) {
        let sel = DeviceSel::from(offset);

        self.internal
            .lock()
            .unwrap()
            .handle_ecam_write(sel.bus, sel.device, sel.func, sel.offset, data);
    }
}
