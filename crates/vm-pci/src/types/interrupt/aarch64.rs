use crate::types::interrupt::InterruptMapEntryArch;

pub struct InterruptMapEntryAArch64 {
    pub pci_addr_high: u32,
    pub pci_addr_mid: u32,
    pub pci_addr_low: u32,
    pub pin: u32,
    pub gic_phandle: u32,
    pub gic_addr_high: u32,
    pub gic_addr_low: u32,
    pub gic_irq_type: u32,
    pub gic_irq_num: u32,
    pub gic_irq_flags: u32,
}

impl InterruptMapEntryArch for InterruptMapEntryAArch64 {
    fn to_vec(&self) -> Vec<u32> {
        [
            self.pci_addr_high,
            self.pci_addr_mid,
            self.pci_addr_low,
            self.pin,
            self.gic_phandle,
            self.gic_addr_high,
            self.gic_addr_low,
            self.gic_irq_type,
            self.gic_irq_num,
            self.gic_irq_flags,
        ]
        .to_vec()
    }
}
