use std::ops::Range;

use tracing::debug;
use tracing::warn;
use vm_core::utils::address_space::AddressSpace;

use crate::device::function::BarHandler;

struct Destination {
    bus: u8,
    device: u8,
    function: u8,
    bar: u8,
    handler: Box<dyn BarHandler>,
}

#[derive(Default)]
pub struct MmioRouter {
    pci_address_space: AddressSpace<u64, Destination>,
}

impl MmioRouter {
    pub fn register_handler(
        &mut self,
        pci_address_range: Range<u64>,
        bus: u8,
        device: u8,
        function: u8,
        bar: u8,
        handler: Box<dyn BarHandler>,
    ) {
        debug!(
            bus,
            device,
            function,
            bar,
            ?pci_address_range,
            "update mmio handler"
        );

        if self
            .pci_address_space
            .try_insert(
                vm_core::utils::address_space::Range {
                    start: pci_address_range.start,
                    len: (pci_address_range.end - pci_address_range.start) as usize,
                },
                Destination {
                    bus,
                    device,
                    function,
                    bar,
                    handler,
                },
            )
            .is_err()
        {
            warn!("remap range: {:?} ignored", pci_address_range);
        }
    }

    pub fn get_handler(&self, pci_address: u64) -> Option<(Range<u64>, &dyn BarHandler)> {
        let (range, dst) = self
            .pci_address_space
            .try_get_value_by_key(pci_address)
            .unwrap();

        debug!(pci_address, dst.bus, dst.device, dst.function, dst.bar);

        Some((
            range.start..range.start + range.len as u64,
            dst.handler.as_ref(),
        ))
    }
}
