use vm_bootloader::linux::bzimage::KERNEL_START;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;

use crate::firmware::bios::e820::*;
use crate::firmware::bios::ivt::InterruptVectorTable;

mod ivt {
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct IvEntry {
        ip: u16,
        cs: u16,
    }

    impl From<u32> for IvEntry {
        fn from(addr: u32) -> Self {
            IvEntry {
                ip: (addr & 0xffff) as u16,
                cs: (addr >> 16) as u16,
            }
        }
    }

    #[repr(C)]
    pub struct InterruptVectorTable {
        entries: [IvEntry; 256],
    }

    impl Default for InterruptVectorTable {
        fn default() -> Self {
            Self {
                entries: [Default::default(); 256],
            }
        }
    }

    impl InterruptVectorTable {
        pub fn set_entry(&mut self, index: u32, addr: u32) {
            self.entries[index as usize] = IvEntry::from(addr);
        }

        pub fn len(&self) -> usize {
            self.entries.len() * 4
        }

        pub fn as_bytes(&self) -> &[u8] {
            unsafe {
                core::slice::from_raw_parts(
                    self as *const _ as *const u8,
                    core::mem::size_of::<InterruptVectorTable>(),
                )
            }
        }
    }
}

mod e820 {
    const E820_X_MAX: usize = 128;

    #[repr(u32)]
    #[derive(Clone, Copy)]
    pub enum E820Type {
        Ram = 1,
        Reserved = 2,
    }

    #[repr(C, packed)]
    #[derive(Clone, Copy, Default)]
    pub struct E820Entry {
        pub addr: u64,
        pub size: u64,
        pub typ: u32,
    }

    #[repr(C, packed)]
    pub struct E820Map {
        nr_map: u32,
        map: [E820Entry; E820_X_MAX],
    }

    impl Default for E820Map {
        fn default() -> Self {
            Self {
                nr_map: Default::default(),
                map: [E820Entry::default(); E820_X_MAX],
            }
        }
    }

    impl E820Map {
        pub fn insert(&mut self, entry: E820Entry) {
            let index = self.nr_map;

            self.map[index as usize] = entry;
            self.nr_map += 1;
        }

        pub fn as_bytes(&self) -> &[u8] {
            unsafe {
                core::slice::from_raw_parts(
                    self as *const _ as *const u8,
                    core::mem::size_of::<E820Map>(),
                )
            }
        }
    }
}

#[derive(Default)]
pub struct Bios;

const BIOS_OFFSET: usize = 0xf000;

impl Bios {
    pub fn init<C>(
        &self,
        memory: &mut MemoryAddressSpace<C>,
        memory_size: usize,
    ) -> anyhow::Result<()>
    where
        C: MemoryContainer,
    {
        let bios_bin = include_bytes!("../../../../bios.bin");
        {
            memory.copy_from_slice(BIOS_OFFSET as u64, bios_bin, bios_bin.len())?;
        }

        {
            let mut ivt = InterruptVectorTable::default();
            for i in 0..256 {
                ivt.set_entry(i, (BIOS_OFFSET + 0x30) as u32);
            }
            ivt.set_entry(0x10, (BIOS_OFFSET + 0x40) as u32);
            ivt.set_entry(0x15, (BIOS_OFFSET + 0x80) as u32);

            memory.copy_from_slice(0, ivt.as_bytes(), ivt.len())?;
        }

        {
            let mut e820 = E820Map::default();
            e820.insert(E820Entry {
                addr: 0,
                size: 4 * 256,
                typ: E820Type::Ram as u32,
            });
            e820.insert(E820Entry {
                addr: BIOS_OFFSET as u64,
                size: bios_bin.len() as u64,
                typ: E820Type::Reserved as u32,
            });
            // trampoline
            e820.insert(E820Entry {
                addr: 0x70000u64,
                size: 0x8000,
                typ: E820Type::Ram as u32,
            });
            e820.insert(E820Entry {
                addr: KERNEL_START as u64,
                size: memory_size as u64 - KERNEL_START as u64,
                typ: E820Type::Ram as u32,
            });

            memory.copy_from_slice(0x0009fc00, e820.as_bytes(), std::mem::size_of::<E820Map>())?;
        }

        Ok(())
    }
}
