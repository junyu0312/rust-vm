/*
 * Refer to:
 * - `https://wiki.osdev.org/Global_Descriptor_Table`
 * - `linux/arch/x86/include/asm/segment.h`
 */

use kvm_bindings::kvm_segment;
use zerocopy::Immutable;
use zerocopy::IntoBytes;

#[repr(packed, C)]
#[derive(Clone, Copy, IntoBytes, Immutable)]
pub struct GdtEntry(u64);

impl GdtEntry {
    pub fn new(base: u32, limit: u32, flags: u32) -> Self {
        let entry = (((base as u64) & 0xff00_0000) << (56 - 24))
            | (((flags as u64) & 0x0000_f0ff) << 40)
            | (((limit as u64) & 0x000f_0000) << (48 - 16))
            | (((base as u64) & 0x00ff_ffff) << 16)
            | ((limit as u64) & 0x0000_ffff);

        GdtEntry(entry)
    }

    pub fn to_kvm_segment(&self, index: u16) -> kvm_segment {
        let entry = self.0;
        let g = ((entry >> 55) & 1) as u8;
        let p = ((entry >> 47) & 1) as u8;
        let limit = ((entry & 0x0000_ffff) | ((entry >> (48 - 16)) & 0x000f_0000)) as u32;
        let limit = match g {
            0 => limit,
            1 => (limit << 12) | 0xfff, // When G=1, the limit is in 4KiB blocks, and the lower 12 bits are ignored.
            _ => unreachable!(),
        };

        kvm_segment {
            base: ((entry >> 16) & 0x00ff_ffff) | ((entry >> (56 - 24)) & 0xff00_0000),
            limit,
            selector: index * 8,
            type_: ((entry >> 40) & 0x0000_000f) as u8, // TODO: 0x0f or 0xff?
            present: p,
            dpl: ((entry >> 45) & 3) as u8,
            db: ((entry >> 54) & 1) as u8,
            s: ((entry >> 44) & 1) as u8,
            l: ((entry >> 53) & 1) as u8,
            g,
            avl: ((entry >> 52) & 1) as u8,
            unusable: match p {
                0 => 1,
                1 => 0,
                _ => unreachable!(),
            },
            padding: 0,
        }
    }
}

#[repr(packed, C)]
#[derive(IntoBytes, Immutable)]
pub struct Gdt<const ENTRY: usize> {
    pub entries: [GdtEntry; ENTRY],
}

impl<const ENTRY: usize> Default for Gdt<ENTRY> {
    fn default() -> Self {
        Gdt {
            entries: [GdtEntry(0); ENTRY],
        }
    }
}

impl<const ENTRY: usize> Gdt<ENTRY> {
    pub fn new(entries: [GdtEntry; ENTRY]) -> Self {
        Gdt { entries }
    }
}
