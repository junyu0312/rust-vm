use crate::arch::aarch64::AArch64;
use crate::virt::kvm::KvmArch;
use crate::virt::kvm::KvmVirt;

impl KvmArch for KvmVirt<AArch64> {
    fn arch_post_init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
