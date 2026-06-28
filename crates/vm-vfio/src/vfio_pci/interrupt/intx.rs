use vm_pci::device::interrupt::legacy::InterruptPin;

pub struct VfioIntxInfo {
    pub pin: InterruptPin,
    pub line: u8,
}

pub struct VfioIntx {
    pub enabled: bool,
}
