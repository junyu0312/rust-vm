#[derive(Default)]
pub struct ConfigAddress(u32);

#[allow(dead_code)]
impl ConfigAddress {
    pub fn write(&mut self, offset: u8, buf: &[u8]) {
        let offset = offset as usize;
        let mut val = self.0.to_le_bytes();
        val[offset..offset + buf.len()].copy_from_slice(buf);
        self.0 = u32::from_le_bytes(val);
    }

    pub fn read(&mut self, offset: u8, buf: &mut [u8]) {
        let offset = offset as usize;
        let bytes = self.0.to_le_bytes();
        buf.copy_from_slice(&bytes[offset..(offset + buf.len())]);
    }

    pub fn enable(&self) -> bool {
        (self.0 & 0x8000_0000) != 0
    }

    pub fn bus(&self) -> u8 {
        ((self.0 >> 16) & 0xff) as u8
    }

    pub fn device(&self) -> u8 {
        ((self.0 >> 11) & 0x1f) as u8
    }

    pub fn function(&self) -> u8 {
        ((self.0 >> 8) & 0x07) as u8
    }

    pub fn register(&self) -> u8 {
        ((self.0 >> 2) & 0x3f) as u8
    }

    pub fn offset(&self) -> u8 {
        (self.0 & 0x3) as u8
    }
}
