#[repr(u16)]
pub enum PciStatus {
    PciStatusImmReady = 0x01,    /* Immediate Readiness */
    PciStatusInterrupt = 0x08,   /* Interrupt status */
    PciStatusCapList = 0x10,     /* Support Capability List */
    PciStatus66mhz = 0x20,       /* Support 66 MHz PCI 2.1 bus */
    PciStatusUdf = 0x40,         /* Support User Definable Features [obsolete] */
    PciStatusFastBack = 0x80,    /* Accept fast-back to back */
    PciStatusParity = 0x100,     /* Detected parity error */
    PciStatusDevselMask = 0x600, /* DEVSEL timing */
    PciStatusDevselFast = 0x000,
    PciStatusDevselMedium = 0x200,
    PciStatusDevselSlow = 0x400,
    PciStatusSigTargetAbort = 0x800,  /* Set on target abort */
    PciStatusRecTargetAbort = 0x1000, /* Master ack of " */
    PciStatusRecMasterAbort = 0x2000, /* Set on master abort */
    PciStatusSigSystemError = 0x4000, /* Set when we drive SERR */
    PciStatusDetectedParity = 0x8000, /* Set on parity error */
}
