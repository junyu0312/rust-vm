use std::slice::Iter;

use crate::device::Error;
use crate::device::Result;
use crate::device::address_space::AddressSpace;
use crate::device::mmio::MmioLayout;
use crate::device::mmio::MmioRange;
use crate::device::mmio::mmio_device::MmioDevice;

pub struct MmioAddressSpaceManager {
    mmio_layout: MmioLayout,
    devices: Vec<Box<dyn MmioDevice>>,
    address_space: AddressSpace<u64>,
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
        let range = device.mmio_range();

        if !self.mmio_layout.includes(range) {
            return Err(Error::InvalidRange);
        }

        let idx = self.devices.len();

        self.address_space.try_insert(range, idx)?;
        self.devices.push(device);

        Ok(())
    }

    pub fn get_device_by_port_mut(
        &mut self,
        addr: u64,
    ) -> Option<(MmioRange, &mut dyn MmioDevice)> {
        let (range, idx) = self.address_space.try_get_value_by_key(addr)?;

        Some((range, self.devices[idx].as_mut()))
    }

    pub fn in_mmio_range(&self, addr: u64) -> bool {
        self.mmio_layout.contains(addr)
    }

    pub fn devices(&self) -> Iter<'_, Box<dyn MmioDevice>> {
        self.devices.iter()
    }
}
