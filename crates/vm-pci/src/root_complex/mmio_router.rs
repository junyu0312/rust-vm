use std::ops::Range;

use rangemap::RangeMap;
use tracing::debug;

#[derive(Clone, PartialEq)]
pub struct Destination {
    pub(crate) bus: u8,
    pub(crate) device: u8,
    pub(crate) function: u8,
    pub(crate) bar: u8,
    pub(crate) pci_address_start: u64,
}

#[derive(Default)]
pub struct MmioRouter {
    pci_address_space: RangeMap<u64, Destination>,
}

impl MmioRouter {
    pub fn register_handler(
        &mut self,
        pci_address_range: Range<u64>,
        bus: u8,
        device: u8,
        function: u8,
        bar: u8,
    ) {
        debug!(
            bus,
            device,
            function,
            bar,
            ?pci_address_range,
            "update mmio handler"
        );

        self.pci_address_space.insert(
            pci_address_range.clone(),
            Destination {
                bus,
                device,
                function,
                bar,
                pci_address_start: pci_address_range.start,
            },
        );
    }

    pub fn get_handler(&self, pci_address: u64) -> Option<Destination> {
        self.pci_address_space.get(&pci_address).cloned()
    }
}
