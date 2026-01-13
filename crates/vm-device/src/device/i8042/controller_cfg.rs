use bitflags::bitflags;

bitflags! {
    pub struct ControllerConfigurationByte: u8 {
        const FirstPs2PortInterrupt = 1 << 0;
        const SecondPs2PortInterrupt = 1 << 1;
        const SystemFlag = 1 << 2;
        const Zero0 = 1 << 3;
        const FirstPs2PortClock = 1 << 4;
        const SecondPs2PortClock = 1 << 5;
        const FirstPs2PortTranslation = 1 << 6;
        const Zero1 = 1 << 7;
    }
}

impl Default for ControllerConfigurationByte {
    fn default() -> Self {
        ControllerConfigurationByte::SystemFlag | ControllerConfigurationByte::FirstPs2PortInterrupt
    }
}
