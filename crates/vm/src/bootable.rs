use crate::kvm::vm::KvmVm;

pub mod linux;

pub trait Bootable {
    fn init(&self, vm: &mut KvmVm) -> anyhow::Result<()>;
}
