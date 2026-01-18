use crate::virt::kvm::KvmVirt;

impl KvmVirt {
    pub fn arch_post_init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
