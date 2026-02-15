use zerocopy::Immutable;
use zerocopy::IntoBytes;

pub mod msi;
pub mod msix;

#[repr(u8)]
pub enum PciCapId {
    Msi = 0x05,  /* Message Signalled Interrupts */
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
