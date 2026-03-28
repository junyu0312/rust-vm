use std::sync::Arc;
use std::sync::Mutex;

use vm_pci::device::function::BarHandler;

use crate::transport::VirtioDev;
use crate::transport::pci::VirtioPciDevice;

pub struct DeviceHandler<D>
where
    D: VirtioPciDevice,
{
    pub dev: Arc<Mutex<VirtioDev<D>>>,
}

impl<D> BarHandler for DeviceHandler<D>
where
    D: VirtioPciDevice,
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
