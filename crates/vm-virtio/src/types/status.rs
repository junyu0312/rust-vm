use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, Default)]
    pub struct Status: u8 {
        // Indicates that the guest OS has found the device
        // and recognized it as a valid virtio device.
        const ACKNOWLEDGE = 1 << 0;

        // Indicates that the guest OS knows how to drive the device.
        const DRIVER = 1 << 1;

        // Indicates that the driver is set up and ready to drive the device.
        const DRIVER_OK = 1 << 2;

        // Indicates that the driver has acknowledged all the features it
        // understands, and feature negotiation is complete.
        const FEATURES_OK = 1 << 3;

        // When VIRTIO_F_SUSPEND is negotiated, indicates that the device
        // has been suspended by the driver.
        const SUSPEND = 1 << 4;

        // Indicates that the device has experienced an error from which
        // it can't recover.
        const DEVICE_NEEDS_RESET = 1 << 6;

        // Indicates that something went wrong in the guest, and it has
        // given up on the device.
        const FAILED = 1 << 7;
    }
}

impl Status {
    pub fn as_u32(&self) -> u32 {
        self.bits() as u32
    }

    pub fn device_needs_reset(&self) -> bool {
        self.contains(Status::DEVICE_NEEDS_RESET)
    }

    pub fn failed(&self) -> bool {
        self.contains(Status::FAILED)
    }
}
