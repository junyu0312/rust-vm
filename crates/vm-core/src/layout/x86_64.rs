use crate::layout::MemoryLayout;
use crate::layout::Result;

#[derive(Clone)]
pub struct X86_64Layout {}

impl MemoryLayout for X86_64Layout {
    fn get_mmio_start(&self) -> u64 {
        todo!()
    }

    fn get_mmio_len(&self) -> usize {
        todo!()
    }

    fn get_ram_base(&self) -> u64 {
        todo!()
    }

    fn set_ram_size(&self, _len: u64) -> Result<()> {
        todo!()
    }

    fn get_ram_size(&self) -> Result<u64> {
        todo!()
    }

    fn set_kernel(&self, _kernel_start: u64, _kernel_len: usize, _start_pc: u64) -> Result<()> {
        todo!()
    }

    fn get_kernel_start(&self) -> Result<u64> {
        todo!()
    }

    fn get_kernel_len(&self) -> Result<usize> {
        todo!()
    }

    fn get_start_pc(&self) -> Result<u64> {
        todo!()
    }

    fn get_initrd_start(&self) -> u64 {
        todo!()
    }

    fn set_initrd_len(&self, _len: usize) -> Result<()> {
        todo!()
    }

    fn get_initrd_len(&self) -> Result<usize> {
        todo!()
    }

    fn get_dtb_start(&self) -> u64 {
        todo!()
    }

    fn set_dtb_len(&self, _len: usize) -> Result<()> {
        todo!()
    }

    fn validate(&self) -> Result<()> {
        todo!()
    }
}
