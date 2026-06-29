use vm_pci::device::interrupt::legacy::InterruptPin;
use vmm_sys_util::eventfd::EventFd;

pub struct VfioIntxInfo {
    pub trigger_fd: EventFd,
    pub resample_fd: EventFd,
    pub pin: InterruptPin,
    pub line: u8,
}

pub struct VfioIntx {
    pub enabled: bool,
}
