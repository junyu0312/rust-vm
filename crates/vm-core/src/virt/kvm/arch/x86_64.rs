use kvm_bindings::kvm_pit_config;

use crate::virt::kvm::KvmVirt;

impl KvmVirt<X86_64> {
    pub fn arch_post_init(&mut self) -> anyhow::Result<()> {
        {
            let pit_config = kvm_pit_config::default();
            self.vm_fd.create_pit2(pit_config).unwrap();
        }

        Ok(())
    }
}
