#[allow(dead_code)]
#[repr(u16)]
pub enum PciStatus {
    ImmReady = 0x01,    /* Immediate Readiness */
    Interrupt = 0x08,   /* Interrupt status */
    CapList = 0x10,     /* Support Capability List */
    Mhz66 = 0x20,       /* Support 66 MHz PCI 2.1 bus */
    Udf = 0x40,         /* Support User Definable Features [obsolete] */
    FastBack = 0x80,    /* Accept fast-back to back */
    Parity = 0x100,     /* Detected parity error */
    DevselMask = 0x600, /* DEVSEL timing */
    DevselFast = 0x000,
    DevselMedium = 0x200,
    DevselSlow = 0x400,
    SigTargetAbort = 0x800,  /* Set on target abort */
    RecTargetAbort = 0x1000, /* Master ack of " */
    RecMasterAbort = 0x2000, /* Set on master abort */
    SigSystemError = 0x4000, /* Set when we drive SERR */
    DetectedParity = 0x8000, /* Set on parity error */
}
