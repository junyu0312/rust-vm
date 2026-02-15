pub mod msi;
pub mod msix;

#[derive(Clone, Copy)]
#[repr(u16)]
pub enum PciCapId {
    Msi = 0x05,  /* Message Signalled Interrupts */
    Vndr = 0x09, /* Vendor-Specific */
    MsiX = 0x11, /* MSI-X */
}
