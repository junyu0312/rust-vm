use thiserror::Error;
use vm_core::interrupt_manager::InterruptManagerError;
use vm_utils::range_allocator::RangeAllocatorError;

#[derive(Error, Debug)]
pub enum VirtioError {
    #[error("Failed to alloc mmio range")]
    AllocMmioRange(RangeAllocatorError),

    #[error("Failed to alloc irq")]
    AllocIrq(#[from] InterruptManagerError),

    #[error("Failed to alloc virtio-mmio id")]
    AllocId(RangeAllocatorError),

    #[error("queue id exceeds u16")]
    QueueExceedsU16 { device: &'static str },

    #[error("Virtio device {device} does not provide a handler for queue {queue_sel}")]
    NoHandlerForVirtqueue {
        device: &'static str,
        queue_sel: u16,
    },

    #[error("Cannot find virtqueue {queue_sel}")]
    VirtqueueNotFound { queue_sel: u16 },

    #[error("Mmio read with invalid buf size")]
    MmioReadInvalidBufSize,

    #[error("Mmio read with invalid register offset")]
    MmioReadInvalidRegisterOffset,

    #[error("Mmio write with invalid buf size")]
    MmioWriteInvalidBufSize,

    #[error("Mmio write with invalid register offset")]
    MmioWriteInvalidRegisterOffset,

    #[error("Mmio write error: {0}")]
    MmioWrite(String),

    #[error("Mmio r/w offset too large")]
    MmioOffsetTooLarge,

    #[error("invalid length of flag")]
    InvalidFlagLen,

    #[error("invalid write device-configuration from driver")]
    DriverWriteDeviceConfigurationInvalid,

    #[error("invalid read device-configuration from driver")]
    DriverReadDeviceConfigurationInvalid,

    #[error("access invalid gpa 0x{0:x}")]
    AccessInvalidGpa(u64),
}

pub type Result<T> = core::result::Result<T, VirtioError>;
