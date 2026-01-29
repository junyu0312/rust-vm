use crate::virtio::device::Subsystem;
use crate::virtio::transport::mmio::VirtIoMmio;

pub trait VirtIOInput: VirtIoMmio {}

impl<T> Subsystem for T
where
    T: VirtIOInput,
{
    const DEVICE_ID: u8 = 18;
}
