#[repr(u8)]
pub enum InterruptPin {
    Empty = 0x00,
    INTA = 0x01,
    INTB = 0x02,
    INTC = 0x03,
    INTD = 0x04,
}
