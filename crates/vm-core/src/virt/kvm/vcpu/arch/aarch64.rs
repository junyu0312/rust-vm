use kvm_ioctls::Kvm;

use crate::device::pio::IoAddressSpace;
use crate::vcpu::Vcpu;
use crate::vcpu::arch::aarch64::AArch64Vcpu;
use crate::vcpu::arch::aarch64::reg::CoreRegister;
use crate::vcpu::arch::aarch64::reg::SysRegister;
use crate::virt::kvm::vcpu::KvmVcpu;

mod encode;

impl KvmVcpu {
    pub fn init_arch_vcpu(&self, _kvm: &Kvm) -> anyhow::Result<()> {
        todo!()
    }
}

impl AArch64Vcpu for KvmVcpu {
    fn get_core_reg(&self, reg: CoreRegister) -> anyhow::Result<u64> {
        let mut bytes = [0; 8];
        let len = self.vcpu_fd.get_one_reg(reg.to_kvm_reg(), &mut bytes)?;
        assert_eq!(len, 8);

        let value = u64::from_le_bytes(bytes);
        Ok(value)
    }

    fn set_core_reg(&self, reg: CoreRegister, value: u64) -> anyhow::Result<()> {
        let len = self
            .vcpu_fd
            .set_one_reg(reg.to_kvm_reg(), &value.to_le_bytes())?;
        assert_eq!(len, 8);

        Ok(())
    }

    fn get_sys_reg(&self, _reg: SysRegister) -> anyhow::Result<u64> {
        todo!()
    }

    fn set_sys_reg(&self, _reg: SysRegister, _value: u64) -> anyhow::Result<()> {
        todo!()
    }
}

impl Vcpu for KvmVcpu {
    fn run(&mut self, _device: &mut IoAddressSpace) -> anyhow::Result<()> {
        todo!()
    }
}
