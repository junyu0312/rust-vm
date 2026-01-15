use bitflags::bitflags;

bitflags! {
    pub struct StatusRegister: u8 {
        const STR_OBF = 1 << 0;
        const STR_IBF = 1 << 1;
        const STR_MUXERR = 1 << 2;
        const STR_CMDDAT = 1 << 3;
        const STR_KEYLOCK = 1 << 4;
        const STR_AUXDATA = 1 << 5;
        const STR_TIMEOUT = 1 << 6;
        const STR_PARITY = 1 << 7;
    }
}

impl Default for StatusRegister {
    fn default() -> Self {
        StatusRegister::STR_KEYLOCK // Avoid linux warning
    }
}
