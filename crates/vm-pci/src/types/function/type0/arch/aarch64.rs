use vm_core::irq::Phandle;
use vm_core::irq::arch::aarch64::GIC_SPI;
use vm_core::irq::arch::aarch64::IRQ_TYPE_LEVEL_HIGH;

use crate::device::function::type0::PciType0Function;
use crate::device::function::type0::Type0Function;
use crate::types::function::PciFunctionArch;
use crate::types::interrupt::InterruptMapEntry;
use crate::types::interrupt::aarch64::InterruptMapEntryAArch64;

impl<T> PciFunctionArch for Type0Function<T>
where
    T: PciType0Function,
{
    fn interrupt_map_entry(&self, bus: u8, device: u8, function: u8) -> Option<InterruptMapEntry> {
        let internal = self.internal.lock().unwrap();

        internal
            .function
            .legacy_interrupt()
            .map(|(irq_line, pin)| InterruptMapEntryAArch64 {
                pci_addr_high: ((bus as u32) << 16)
                    | ((device as u32) << 11)
                    | ((function as u32) << 8),
                pci_addr_mid: 0,
                pci_addr_low: 0,
                pin: pin.into(),
                gic_phandle: Phandle::GIC as u32,
                gic_addr_high: 0,
                gic_addr_low: 0,
                gic_irq_type: GIC_SPI,
                gic_irq_num: irq_line.into(),
                gic_irq_flags: IRQ_TYPE_LEVEL_HIGH,
            })
    }
}
