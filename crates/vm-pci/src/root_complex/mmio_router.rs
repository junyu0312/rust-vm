use tracing::debug;
use tracing::warn;
use vm_core::device::mmio::layout::MmioRange;
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
        pci_address_range: MmioRange,
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
                pci_address_range,
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

    pub fn get_handler(&self, pci_address: u64) -> Option<(MmioRange, &dyn BarHandler)> {
        let (range, dst) = self
            .pci_address_space
            .try_get_value_by_key(pci_address)
            .unwrap();

        debug!(pci_address, dst.bus, dst.device, dst.function, dst.bar);

        Some((range, dst.handler.as_ref()))
    }
}
