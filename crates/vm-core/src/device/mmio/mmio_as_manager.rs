use std::slice::Iter;

use crate::device::Error;
use crate::device::Result;
use crate::device::address_space::AddressSpace;
use crate::device::mmio::MmioLayout;
use crate::device::mmio::MmioRange;
use crate::device::mmio::mmio_device::MmioDevice;
use crate::device::mmio::mmio_device::MmioHandler;

pub struct MmioAddressSpaceManager {
    mmio_layout: MmioLayout,
    devices: Vec<Box<dyn MmioDevice>>,
    address_space: AddressSpace<u64, Box<dyn MmioHandler>>,
}

impl MmioAddressSpaceManager {
    pub fn new(mmio_layout: MmioLayout) -> Self {
        MmioAddressSpaceManager {
            mmio_layout,
            devices: Default::default(),
            address_space: AddressSpace::new(),
        }
    }

    pub fn register(&mut self, device: Box<dyn MmioDevice>) -> Result<()> {
        let ranges = device.mmio_range_handlers();

        for range in &ranges {
            let mmio_range = range.mmio_range();

            if !self.mmio_layout.includes(mmio_range) {
                return Err(Error::InvalidRange(mmio_range.start, mmio_range.len));
            }

            if self
                .address_space
                .is_overlap(mmio_range.start, mmio_range.len)
            {
                return Err(Error::InvalidRange(mmio_range.start, mmio_range.len));
            }
        }

        for range in ranges {
            let mmio_range = range.mmio_range();

            self.address_space.try_insert(mmio_range, range)?;
        }

        self.devices.push(device);

        Ok(())
    }

    pub fn get_handler_by_addr(&self, addr: u64) -> Option<(MmioRange, &dyn MmioHandler)> {
        let (range, handler) = self.address_space.try_get_value_by_key(addr)?;

        Some((range, handler.as_ref()))
    }

    pub fn in_mmio_range(&self, addr: u64) -> bool {
        self.mmio_layout.contains(addr)
    }

    pub fn devices(&self) -> Iter<'_, Box<dyn MmioDevice>> {
        self.devices.iter()
    }
}
