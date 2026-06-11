use crate::transport::control_register::ControlRegister;
use crate::transport::pci::VirtioPciDevice;
use crate::transport::pci::VirtioPciTransport;

impl<D> VirtioPciTransport<D>
where
    D: VirtioPciDevice,
{
    pub fn read_notify(&self, _offset: u64, _data: &mut [u8]) {
        unreachable!()
    }

    pub fn write_notify(&self, _offset: u64, data: &[u8]) {
        assert_eq!(data.len(), 2);
        let queue_index = u16::from_le_bytes(data.try_into().unwrap());
        let mut dev = self.dev.lock().unwrap();
        dev.write_reg(ControlRegister::QueueNotify, queue_index.into())
            .unwrap();
    }
}
