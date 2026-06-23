use vm_pci::device::interrupt::legacy::InterruptPin;

pub struct VfioIntx {
    pub pin: InterruptPin,
    pub line: u8,
}
