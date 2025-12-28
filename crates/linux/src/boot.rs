pub enum VariantSizeValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}

pub struct BootHeaderEntry {
    pub offset: u64,
    pub size: u64,
    pub value: VariantSizeValue,
}

pub trait ArchBootParams {
    fn as_mut_slice(&mut self) -> &mut [u8];

    fn reset(&mut self);

    fn write(&mut self, entry: &BootHeaderEntry, value: VariantSizeValue) {
        let mem = self.as_mut_slice();
        let offset = entry.offset as usize;
        match value {
            VariantSizeValue::U8(v) => mem[offset] = v,
            VariantSizeValue::U16(v) => {
                let bytes = v.to_le_bytes();
                mem[offset..offset + 2].copy_from_slice(&bytes);
            }
            VariantSizeValue::U32(v) => {
                let bytes = v.to_le_bytes();
                mem[offset..offset + 4].copy_from_slice(&bytes);
            }
            VariantSizeValue::U64(v) => {
                let bytes = v.to_le_bytes();
                mem[offset..offset + 8].copy_from_slice(&bytes);
            }
        }
    }
}
