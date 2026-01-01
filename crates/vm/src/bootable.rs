use crate::kvm::vm::KvmVm;

pub mod linux;

pub trait Bootable {
    fn init(&mut self, vm: &mut KvmVm) -> anyhow::Result<()>;
}
