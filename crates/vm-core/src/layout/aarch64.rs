use std::cell::OnceCell;

use static_assertions::const_assert_eq;

use crate::layout::Error;
use crate::layout::MemoryLayout;
use crate::layout::Result;

const MMIO_START: u64 = 0x0900_0000;
const MMIO_LEN: usize = 0x2000_0000;
const GIC_DISTRIBUTOR: u64 = 0x3000_0000;
const GIC_REDISTRIBUTOR: u64 = 0x3001_0000;
const RAM_BASE: u64 = 0x4000_0000;
const DTB_START: u64 = 0x4400_0000; // Reserve 64MB for kernel
const INITRD_START: u64 = 0x44200000; // DTB + 2MB

const KERNEL_MAX: usize = 0x0400_0000;

const_assert_eq!(RAM_BASE + KERNEL_MAX as u64, DTB_START);

pub struct AArch64Layout {
    mmio_start: u64,
    mmio_len: usize,
    distributor_start: u64,
    distributor_len: OnceCell<usize>,
    redistributor_start: u64,
    redistributor_region_len: OnceCell<usize>,
    ram_base: u64,
    ram_size: OnceCell<u64>,
    kernel_start: OnceCell<u64>,
    kernel_len: OnceCell<usize>,
    dtb_start: u64,
    dtb_len: OnceCell<usize>,
    initrd_start: u64,
    initrd_len: OnceCell<usize>,
    start_pc: OnceCell<u64>,
}

impl Default for AArch64Layout {
    fn default() -> AArch64Layout {
        AArch64Layout {
            mmio_start: MMIO_START,
            mmio_len: MMIO_LEN,
            distributor_start: GIC_DISTRIBUTOR,
            distributor_len: OnceCell::new(),
            redistributor_start: GIC_REDISTRIBUTOR,
            redistributor_region_len: OnceCell::new(),
            ram_base: RAM_BASE,
            ram_size: OnceCell::new(),
            kernel_start: OnceCell::new(),
            kernel_len: OnceCell::new(),
            dtb_start: DTB_START,
            dtb_len: OnceCell::new(),
            initrd_start: INITRD_START,
            initrd_len: OnceCell::new(),
            start_pc: OnceCell::new(),
        }
    }
}

impl AArch64Layout {
    pub fn get_distributor_start(&self) -> u64 {
        self.distributor_start
    }

    pub fn set_distributor_len(&self, len: usize) -> Result<()> {
        self.distributor_len
            .set(len)
            .map_err(|_| Error::ArchAlreadySet("distributor len".to_string()))
    }

    pub fn get_redistributor_start(&self) -> u64 {
        self.redistributor_start
    }

    pub fn set_redistributor_region_len(&self, len: usize) -> Result<()> {
        self.redistributor_region_len
            .set(len)
            .map_err(|_| Error::ArchAlreadySet("redistributor region len".to_string()))
    }
}

impl MemoryLayout for AArch64Layout {
    fn get_mmio_start(&self) -> u64 {
        self.mmio_start
    }

    fn get_mmio_len(&self) -> usize {
        self.mmio_len
    }

    fn get_ram_base(&self) -> u64 {
        self.ram_base
    }

    fn set_ram_size(&self, len: u64) -> Result<()> {
        self.ram_size.set(len).map_err(|_| Error::RamSizeAlreadySet)
    }

    fn get_ram_size(&self) -> Result<u64> {
        Ok(*self.ram_size.get().ok_or(Error::RamSizeUnset)?)
    }

    fn set_kernel(&self, kernel_start: u64, kernel_len: usize, start_pc: u64) -> Result<()> {
        if self.kernel_start.get().is_some()
            || self.kernel_len.get().is_some()
            || self.start_pc.get().is_some()
        {
            return Err(Error::KernelAlreadySet);
        }

        if kernel_len > KERNEL_MAX {
            return Err(Error::KernelTooLarge);
        }

        self.kernel_start.set(kernel_start).unwrap();
        self.kernel_len.set(kernel_len).unwrap();
        self.start_pc.set(start_pc).unwrap();

        Ok(())
    }

    fn get_kernel_start(&self) -> super::Result<u64> {
        Ok(*self.kernel_start.get().ok_or(Error::KernelUnset)?)
    }

    fn get_kernel_len(&self) -> Result<usize> {
        Ok(*self.kernel_len.get().ok_or(Error::KernelUnset)?)
    }

    fn get_start_pc(&self) -> Result<u64> {
        Ok(*self.start_pc.get().ok_or(Error::KernelUnset)?)
    }

    fn get_initrd_start(&self) -> u64 {
        self.initrd_start
    }

    fn set_initrd_len(&self, len: usize) -> Result<()> {
        self.initrd_len
            .set(len)
            .map_err(|_| Error::InitrdAlreadySet)
    }

    fn get_initrd_len(&self) -> Result<usize> {
        Ok(*self.initrd_len.get().ok_or(Error::InitrdUnset)?)
    }

    fn get_dtb_start(&self) -> u64 {
        self.dtb_start
    }

    fn set_dtb_len(&self, len: usize) -> Result<()> {
        self.dtb_len.set(len).map_err(|_| Error::DtbAlreadySet)
    }

    fn validate(&self) -> Result<()> {
        Ok(())
    }
}
