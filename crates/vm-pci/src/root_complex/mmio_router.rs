use vm_core::device::address_space::AddressSpace;
use vm_core::device::mmio::MmioRange;

use crate::types::function::BarHandler;

struct Destination {
    _bus: u8,
    _device: u8,
    _function: u8,
    _bar: u8,
    handler: Box<dyn BarHandler>,
}

pub struct MmioRouter {
    pci_address_space: AddressSpace<u64, Destination>,
}

impl Default for MmioRouter {
    fn default() -> Self {
        Self {
            pci_address_space: AddressSpace::new(),
        }
    }
}

impl MmioRouter {
    pub fn register_handler(
        &mut self,
        bar_range: MmioRange,
        bus: u8,
        device: u8,
        function: u8,
        bar: u8,
        handler: Box<dyn BarHandler>,
    ) {
        if self
            .pci_address_space
            .try_insert(
                bar_range,
                Destination {
                    _bus: bus,
                    _device: device,
                    _function: function,
                    _bar: bar,
                    handler,
                },
            )
            .is_err()
        {
            println!("remap range: {:?} ignored", bar_range);
        }
    }

    pub fn get_handler(&self, offset: u64) -> Option<&dyn BarHandler> {
        Some(
            self.pci_address_space
                .try_get_value_by_key(offset)
                .unwrap()
                .1
                .handler
                .as_ref(),
        )
    }
}
