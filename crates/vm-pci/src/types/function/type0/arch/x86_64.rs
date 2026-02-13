use crate::device::function::type0::PciType0Function;
use crate::device::function::type0::Type0Function;
use crate::types::function::PciFunctionArch;
use crate::types::interrupt::InterruptMapEntry;
use crate::types::interrupt::x86_64::InterruptMapEntryX86_64;

impl<T> PciFunctionArch for Type0Function<T>
where
    T: PciType0Function,
{
    fn interrupt_map_entry(
        &self,
        _bus: u8,
        _device: u8,
        _function: u8,
    ) -> Option<InterruptMapEntry> {
        let internal = self.internal.lock().unwrap();

        internal
            .function
            .legacy_interrupt()
            .map(|(_irq_line, _pin)| InterruptMapEntryX86_64 {})
    }
}
