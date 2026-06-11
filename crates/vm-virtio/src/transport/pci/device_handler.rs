use crate::transport::pci::VirtioPciDevice;
use crate::transport::pci::VirtioPciTransport;

impl<D> VirtioPciTransport<D>
where
    D: VirtioPciDevice,
{
    pub fn read_device(&self, offset: u64, data: &mut [u8]) {
        let dev = self.dev.lock().unwrap();

        dev.read_config(offset.try_into().unwrap(), data).unwrap();
    }

    pub fn write_device(&self, offset: u64, data: &[u8]) {
        let mut dev = self.dev.lock().unwrap();

        dev.write_config(offset.try_into().unwrap(), data).unwrap();
    }
}
