pub mod virtio_input;

pub trait Subsystem {
    const DEVICE_ID: u8;
}
