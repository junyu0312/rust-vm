use bitflags::bitflags;

bitflags! {
    pub struct StatusRegister: u8 {
        const OBF = 1 << 0;
        const IBF = 1 << 1;
        const SystemFlag = 1 << 2;
        const Command = 1 << 3;
        const Keylock = 1 << 4;
        const AuxData = 1 << 5;
        const TimeoutError = 1 << 6;
        const ParityError = 1 << 7;
    }
}

impl Default for StatusRegister {
    fn default() -> Self {
        let mut reg = StatusRegister::empty();
        reg.insert(StatusRegister::SystemFlag);
        reg
    }
}
