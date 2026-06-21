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
        self.virtqueue_handlers
            .read()
            .unwrap()
            .get(&queue_index)
            .unwrap()
            .controller
            .queue_notify
            .notify_one();
    }
}
