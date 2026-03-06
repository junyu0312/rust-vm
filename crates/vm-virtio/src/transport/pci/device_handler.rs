use std::sync::Arc;
use std::sync::Mutex;

use vm_mm::allocator::MemoryContainer;
use vm_pci::device::function::BarHandler;

use crate::device::pci::VirtioPciDevice;
use crate::transport::VirtioDev;

pub struct DeviceHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    pub dev: Arc<Mutex<VirtioDev<C, D>>>,
}

impl<C, D> BarHandler for DeviceHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    fn read(&self, offset: u64, data: &mut [u8]) {
        let dev = self.dev.lock().unwrap();

        dev.read_config(offset.try_into().unwrap(), data).unwrap();
    }

    fn write(&self, offset: u64, data: &[u8]) {
        let mut dev = self.dev.lock().unwrap();

        dev.write_config(offset.try_into().unwrap(), data).unwrap();
    }
}
