use kvm_bindings::CpuId;

pub fn update_cpuid(cpuid: &CpuId, vcpu_id: u8) -> CpuId {
    let mut cpuid = cpuid.clone();

    for entry in cpuid.as_mut_slice() {
        match entry.function {
            // Version and Features
            0x01 => {
                entry.ebx &= 0xffffff;
                // Update INITIAL_APIC_ID
                entry.ebx |= (vcpu_id as u32) << 24;
            }
            _ => continue,
        }
    }

    cpuid
}
