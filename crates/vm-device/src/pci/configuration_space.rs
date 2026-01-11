const PCI_CLASS_DEVICE: u8 = 0x0a;

pub struct ConfigurationSpace {
    buf: [u8; 256],
}

impl ConfigurationSpace {
    pub fn new(device_class: u16) -> Self {
        let mut buf = [0; 256];
        buf[(PCI_CLASS_DEVICE as usize)..(PCI_CLASS_DEVICE as usize + 2)]
            .copy_from_slice(&device_class.to_le_bytes());

        Self { buf }
    }
}

impl ConfigurationSpace {
    pub fn read(&self, start: u8, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.buf[start as usize..start as usize + bytes.len()]);
    }

    pub fn write(&mut self, start: u8, buf: &[u8]) {
        self.buf[start as usize..start as usize + buf.len()].copy_from_slice(buf);
    }
}
