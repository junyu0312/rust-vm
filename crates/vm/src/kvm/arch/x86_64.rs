use kvm_bindings::kvm_pit_config;

use crate::firmware::bios::Bios;
use crate::kvm::vm::KvmVm;

mod vcpu;

impl KvmVm {
    pub fn init_arch(&mut self) -> anyhow::Result<()> {
        {
            let bios = Bios;
            bios.init(self)?;
        }

        {
            let pit_config = kvm_pit_config::default();
            self.vm_fd.create_pit2(pit_config).unwrap();
        }

        Ok(())
    }
}
