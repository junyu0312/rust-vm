use zerocopy::IntoBytes;

use crate::transport::pci::VirtioPciDevice;
use crate::transport::pci::VirtioPciTransport;
use crate::transport::pci::msix::VirtioPciMsixInfo;

impl<D> VirtioPciTransport<D>
where
    D: VirtioPciDevice,
{
    fn read_table(&self, msix: &VirtioPciMsixInfo, offset: u64, data: &mut [u8]) {
        data.copy_from_slice(&msix.table.as_bytes()[offset as usize..offset as usize + data.len()]);
    }

    fn write_table(&self, msix: &mut VirtioPciMsixInfo, offset: u64, data: &[u8]) {
        msix.table.as_mut_bytes()[offset as usize..offset as usize + data.len()]
            .copy_from_slice(data);
    }

    fn read_pba(&self, _msix: &VirtioPciMsixInfo, _offset: u64, _data: &mut [u8]) {
        todo!()
    }

    fn write_pba(&self, _msix: &VirtioPciMsixInfo, _offset: u64, _data: &[u8]) {
        todo!()
    }

    pub fn read_msix(&self, offset: u64, data: &mut [u8]) {
        let msix = self
            .interrupt_dispatcher
            .msix
            .as_ref()
            .unwrap()
            .read()
            .unwrap();

        if offset < msix.pba_offset() as u64 {
            self.read_table(&msix, offset, data);
        } else {
            self.read_pba(&msix, offset, data);
        }
    }

    pub fn write_msix(&self, offset: u64, data: &[u8]) {
        let mut msix = self
            .interrupt_dispatcher
            .msix
            .as_ref()
            .unwrap()
            .write()
            .unwrap();

        if offset < msix.pba_offset() as u64 {
            self.write_table(&mut msix, offset, data);
        } else {
            self.write_pba(&msix, offset, data);
        }
    }
}
