pub mod msix;

#[repr(u8)]
pub enum PciCapId {
    Vndr = 0x09, /* Vendor-Specific */
    MsiX = 0x11, /* MSI-X */
}
