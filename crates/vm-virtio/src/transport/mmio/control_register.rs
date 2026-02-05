use strum_macros::FromRepr;

#[derive(Debug, FromRepr)]
#[repr(u16)]
pub enum MmioControlRegister {
    /* Control registers */
    /// Magic value ("virt") - Read Only
    MagicValue = 0x000,

    /// Virtio device version - Read Only
    Version = 0x004,

    /// Virtio device ID - Read Only
    DeviceId = 0x008,

    /// Virtio vendor ID - Read Only
    VendorId = 0x00c,

    /// Device features (host) - Read Only
    DeviceFeatures = 0x010,

    /// Device features selector - Write Only
    DeviceFeaturesSel = 0x014,

    /// Driver features (guest) - Write Only
    DriverFeatures = 0x020,

    /// Driver features selector - Write Only
    DriverFeaturesSel = 0x024,

    /// Queue selector - Write Only
    QueueSel = 0x030,

    /// Maximum queue size - Read Only
    QueueSizeMax = 0x034,

    /// Queue size - Write Only
    QueueSize = 0x038,

    /// Queue ready - Read Write
    QueueReady = 0x044,

    /// Queue notify - Write Only
    QueueNotify = 0x050,

    /// Interrupt status - Read Only
    InterruptStatus = 0x060,

    /// Interrupt acknowledge - Write Only
    InterruptAck = 0x064,

    /// Device status - Read Write
    Status = 0x070,

    /// Descriptor table address (low 32 bits)
    QueueDescLow = 0x080,

    /// Descriptor table address (high 32 bits)
    QueueDescHigh = 0x084,

    /// Available ring address (low 32 bits)
    QueueAvailLow = 0x090,

    /// Available ring address (high 32 bits)
    QueueAvailHigh = 0x094,

    /// Used ring address (low 32 bits)
    QueueUsedLow = 0x0a0,

    /// Used ring address (high 32 bits)
    QueueUsedHigh = 0x0a4,

    /// Shared memory region selector
    ShmSel = 0x0ac,

    /// Shared memory length (low 32 bits)
    ShmLenLow = 0x0b0,

    /// Shared memory length (high 32 bits)
    ShmLenHigh = 0x0b4,

    /// Shared memory base address (low 32 bits)
    ShmBaseLow = 0x0b8,

    /// Shared memory base address (high 32 bits)
    ShmBaseHigh = 0x0bc,

    /// Queue reset
    QueueReset = 0x0c0,

    /// Configuration generation
    ConfigGeneration = 0x0fc,
}
