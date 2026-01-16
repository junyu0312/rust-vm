use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct ControllerConfigurationByte: u8 {
        const CTL_KBDINT = 1 << 0;
        const CTL_AUXINT = 1 << 1;
        const CTL_IGNKEYLOCK = 1 << 3;
        const CTL_KBDDIS = 1 << 4;
        const CTL_AUXDIS = 1 << 5;
        const CTL_XLATE = 1 << 6;
    }
}
