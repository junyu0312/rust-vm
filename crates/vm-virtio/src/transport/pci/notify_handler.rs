use std::sync::Arc;
use std::sync::Mutex;

use vm_pci::device::function::BarHandler;

use crate::transport::VirtioDev;
use crate::transport::control_register::ControlRegister;
use crate::transport::pci::VirtioPciDevice;

pub struct NotifyHandler<D>
where
    D: VirtioPciDevice,
{
    pub dev: Arc<Mutex<VirtioDev<D>>>,
}

impl<D> BarHandler for NotifyHandler<D>
where
    D: VirtioPciDevice,
{
    fn read(&self, _offset: u64, _data: &mut [u8]) {
        unreachable!()
    }

    fn write(&self, _offset: u64, data: &[u8]) {
        assert_eq!(data.len(), 2);
        let queue_index = u16::from_le_bytes(data.try_into().unwrap());
        let mut dev = self.dev.lock().unwrap();
        dev.write_reg(ControlRegister::QueueNotify, queue_index.into())
            .unwrap();
    }
}
