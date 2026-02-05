#[derive(Debug)]
pub enum ControlRegister {
    /// Device features (host)
    DeviceFeatures,

    /// Device features selector
    DeviceFeaturesSel,

    /// Driver features (guest)
    DriverFeatures,

    /// Driver features selector
    DriverFeaturesSel,

    /// Queue selector
    QueueSel,

    /// Maximum queue size
    QueueSizeMax,

    /// Queue size
    QueueSize,

    /// Queue ready
    QueueReady,

    /// Queue notify
    QueueNotify,

    /// Interrupt status
    InterruptStatus,

    /// Device status
    Status,

    /// Descriptor table address (low 32 bits)
    QueueDescLow,

    /// Descriptor table address (high 32 bits)
    QueueDescHigh,

    /// Available ring address (low 32 bits)
    QueueAvailLow,

    /// Available ring address (high 32 bits)
    QueueAvailHigh,

    /// Used ring address (low 32 bits)
    QueueUsedLow,

    /// Used ring address (high 32 bits)
    QueueUsedHigh,

    /// Shared memory region selector
    ShmSel,

    /// Shared memory length (low 32 bits)
    ShmLenLow,

    /// Shared memory length (high 32 bits)
    ShmLenHigh,

    /// Shared memory base address (low 32 bits)
    ShmBaseLow,

    /// Shared memory base address (high 32 bits)
    ShmBaseHigh,

    /// Queue reset
    QueueReset,

    /// Configuration generation
    ConfigGeneration,
}
