use zerocopy::Immutable;
use zerocopy::IntoBytes;

pub mod msix;

#[repr(u8)]
pub enum PciCapId {
    Vndr = 0x09, /* Vendor-Specific */
    MsiX = 0x11, /* MSI-X */
}

pub struct Capability(pub Vec<u8>);

impl<T> From<T> for Capability
where
    T: IntoBytes + Immutable,
{
    fn from(cap: T) -> Self {
        Capability(cap.as_bytes().to_vec())
    }
}
