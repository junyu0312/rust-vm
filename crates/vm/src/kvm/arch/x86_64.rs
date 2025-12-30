use kvm_bindings::CpuId;
use kvm_bindings::KVM_MAX_CPUID_ENTRIES;

use crate::kvm::vm::KvmVm;

impl KvmVm {
    pub fn get_supported_cpuid(&self) -> anyhow::Result<CpuId> {
        let cpuid = self.kvm.get_supported_cpuid(KVM_MAX_CPUID_ENTRIES)?;

        Ok(cpuid)
    }
}
