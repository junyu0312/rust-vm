pub struct ConfigurationSpace {
    buf: [u8; 256],
}

impl ConfigurationSpace {
    pub fn read(&self, start: u8, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.buf[start as usize..start as usize + bytes.len()]);
    }

    pub fn write(&mut self, start: u8, buf: &[u8]) {
        self.buf[start as usize..start as usize + buf.len()].copy_from_slice(buf);
    }
}
