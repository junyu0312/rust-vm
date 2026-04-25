use std::u64;

use vm_aarch64::register::id_aa64mmfr0_el1::IdAa64mmfr0El1;
use vm_aarch64::register::tcr_el1::TcrEl1;
use vm_aarch64::register::ttbr1_el1::Ttbr1El1;
use vm_mm::manager::MemoryAddressSpace;

use crate::cpu::error::VcpuError;

fn walk(
    mm: &MemoryAddressSpace,
    gva: u64,
    index_of_gva: impl Fn(usize) -> u64,
    granule_size_bits: u8,
    descriptor_addr_mask: u64,
    baddr: u64,
    current_level: usize,
    maximal_level: usize,
) -> Result<u64, VcpuError> {
    assert!(current_level < maximal_level);

    let index = index_of_gva(current_level);
    let descriptor_addr = (baddr | (index << granule_size_bits)) & !0b11;

    let descriptor;
    {
        let mut buf = [0; 8];
        for (offset, b) in buf.iter_mut().enumerate() {
            let hva = mm
                .gpa_to_hva(descriptor_addr + offset as u64)
                .map_err(|_| VcpuError::TranslateErr)?;
            *b = unsafe { *hva };
        }
        descriptor = u64::from_le_bytes(buf);
    }

    let next_addr = descriptor & descriptor_addr_mask;

    if (descriptor & 0b11) == 0b11 && current_level != 3 {
        // table descriptor

        walk(
            mm,
            gva,
            index_of_gva,
            granule_size_bits,
            descriptor_addr_mask,
            next_addr,
            current_level + 1,
            maximal_level,
        )
    } else {
        // block descriptor or page descriptor

        let page_or_block_size =
            1 << ((granule_size_bits - 3) as u64 * (maximal_level - current_level) as u64 + 3);

        let mut gpa = next_addr;
        gpa &= !(page_or_block_size - 1); // Mask flags
        gpa |= gva & (page_or_block_size - 1);
        Ok(gpa)
    }
}

pub fn translate_gva_to_gpa(
    mm: &MemoryAddressSpace,
    tcr_el1: impl FnOnce() -> Result<TcrEl1, VcpuError>,
    ttbr1_el1: impl FnOnce() -> Result<Ttbr1El1, VcpuError>,
    id_aa64mmfr0_el1: impl FnOnce() -> Result<IdAa64mmfr0El1, VcpuError>,
    gva: u64,
) -> Result<u64, VcpuError> {
    let tcr_el1 = tcr_el1()?;
    let ttbr1_el1 = ttbr1_el1()?;
    let id_aa64mmfr0_el1 = id_aa64mmfr0_el1()?;

    if ttbr1_el1.0 == 0 {
        return Err(VcpuError::TranslateErr);
    }

    // println!(
    //     "tcr_el1: {:x} ttbr1_el1: {:x}, id_aa64mmfr0_el1: {:x}",
    //     tcr_el1.0, ttbr1_el1.0, id_aa64mmfr0_el1.0
    // );

    let t1sz = tcr_el1.t1sz();
    let va_size = 64 - t1sz;
    let tg1 = tcr_el1.tg1();

    let ds = tcr_el1.ds();

    let ips = tcr_el1.ips();
    let pa_range = id_aa64mmfr0_el1.pa_range();
    assert_eq!(ips, pa_range);

    let pa_range = match pa_range {
        0b0000 => 32,
        0b0001 => 36,
        0b0010 => 40,
        0b0011 => 42,
        0b0100 => 44,
        0b0101 => 48,
        0b0110 => 52,
        0b0111 => 56,
        _ => return Err(VcpuError::TranslateErr),
    };

    let granule_size_bits = match tg1 {
        0b01 => 14, // 16KB
        0b10 => 12, // 4KB
        0b11 => 16, // 64KB
        _ => return Err(VcpuError::TranslateErr),
    };
    let index_bits = granule_size_bits - 3;
    let index_mask = (1 << index_bits) - 1;

    let descriptor_addr_mask = if ds {
        u64::MAX >> (64 - 50)
    } else {
        u64::MAX >> (64 - 48)
    } & !(u64::MAX >> (64 - granule_size_bits));

    let baddr = ttbr1_el1.baddr();
    // If FEAT_LPA is implemented and the value of TCR_EL1.IPS is 0b110, then:
    if pa_range == 52 {
        // ttbr_baddr |= ((ttbr >> 2) & 0b1111) >> 48;
        todo!()
    }

    let level = 4 - (va_size - 4) / index_bits;

    let index_of_gva = |level: usize| {
        let skip_level = 4 - 1 - level;
        ((gva >> granule_size_bits) >> (index_bits as u64 * skip_level as u64)) & index_mask
    };

    walk(
        mm,
        gva,
        index_of_gva,
        granule_size_bits,
        descriptor_addr_mask,
        baddr,
        level as usize,
        4,
    )
}
