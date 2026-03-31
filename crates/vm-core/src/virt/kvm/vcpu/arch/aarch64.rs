use kvm_ioctls::Kvm;

use crate::arch::aarch64::AArch64;
use crate::arch::aarch64::vcpu::AArch64Vcpu;
use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::arch::aarch64::vm_exit::VmExitReason;
use crate::arch::vcpu::Vcpu;
use crate::error::Result;
use crate::virt::DeviceVmExitHandler;
use crate::virt::kvm::vcpu::KvmVcpu;

mod encode;

impl KvmVcpu {
    pub fn init_arch_vcpu(&self, _kvm: &Kvm) -> Result<()> {
        todo!()
    }
}

impl AArch64Vcpu for KvmVcpu {
    fn get_core_reg(&self, reg: CoreRegister) -> Result<u64> {
        let mut bytes = [0; 8];
        let len = self.vcpu_fd.get_one_reg(reg.to_kvm_reg(), &mut bytes)?;
        assert_eq!(len, 8);

        let value = u64::from_le_bytes(bytes);
        Ok(value)
    }

    fn set_core_reg(&self, reg: CoreRegister, value: u64) -> Result<()> {
        let len = self
            .vcpu_fd
            .set_one_reg(reg.to_kvm_reg(), &value.to_le_bytes())?;
        assert_eq!(len, 8);

        Ok(())
    }

    fn get_sys_reg(&self, _reg: SysRegister) -> Result<u64> {
        todo!()
    }

    fn set_sys_reg(&self, _reg: SysRegister, _value: u64) -> Result<()> {
        todo!()
    }
}

impl Vcpu<AArch64> for KvmVcpu {
    fn run(&mut self, _device_vm_exit_handler: &dyn DeviceVmExitHandler) -> Result<VmExitReason> {
        todo!()
    }
}
