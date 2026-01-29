use kvm_bindings::kvm_pit_config;

use crate::arch::x86_64::X86_64;
use crate::virt::kvm::KvmArch;
use crate::virt::kvm::KvmVirt;

impl KvmArch for KvmVirt<X86_64> {
    fn arch_post_init(&mut self) -> anyhow::Result<()> {
        {
            let pit_config = kvm_pit_config::default();
            self.vm_fd.create_pit2(pit_config).unwrap();
        }

        Ok(())
    }
}
