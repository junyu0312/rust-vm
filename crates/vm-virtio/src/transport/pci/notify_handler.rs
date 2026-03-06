use std::sync::Arc;
use std::sync::Mutex;

use vm_mm::allocator::MemoryContainer;
use vm_pci::device::function::BarHandler;

use crate::device::pci::VirtioPciDevice;
use crate::transport::VirtioDev;
use crate::transport::control_register::ControlRegister;

pub struct NotifyHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    pub dev: Arc<Mutex<VirtioDev<C, D>>>,
}

impl<C, D> BarHandler for NotifyHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
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
