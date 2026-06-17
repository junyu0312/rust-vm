pub const PCI_BASE_ADDRESS_SPACE: u32 = 0x01;
pub const PCI_BASE_ADDRESS_MEM_TYPE_MASK: u32 = 0x06;
pub const PCI_BASE_ADDRESS_MEM_TYPE_32: u32 = 0x00; /* 32 bit address */
pub const PCI_BASE_ADDRESS_MEM_TYPE_1M: u32 = 0x02; /* Below 1M [obsolete] */
pub const PCI_BASE_ADDRESS_MEM_TYPE_64: u32 = 0x04; /* 64 bit address */

const PCI_BASE_ADDRESS_IO_MASK: u32 = !0x03;
const PCI_BASE_ADDRESS_MEMORY_MASK: u32 = !0x0f;

pub fn is_pio_bar(bar: u32) -> bool {
    bar & PCI_BASE_ADDRESS_SPACE != 0
}

pub fn is_mmio_bar(bar: u32) -> bool {
    bar & PCI_BASE_ADDRESS_SPACE == 0
}

pub fn address_of_bar(bar: u32) -> u32 {
    if is_pio_bar(bar) {
        bar & PCI_BASE_ADDRESS_IO_MASK
    } else {
        bar & PCI_BASE_ADDRESS_MEMORY_MASK
    }
}
