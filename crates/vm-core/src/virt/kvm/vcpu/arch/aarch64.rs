use kvm_ioctls::Kvm;

use crate::arch::aarch64::vcpu::AArch64Vcpu;
use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::arch::aarch64::vm_exit::VmExitReason;
use crate::cpu::error::VcpuError;
use crate::virt::kvm::vcpu::KvmVcpu;
use crate::virt::vcpu::HypervisorVcpu;

mod encode;

impl KvmVcpu {
    pub fn init_arch_vcpu(&mut self, _kvm: &Kvm) -> Result<(), VcpuError> {
        todo!()
    }
}

impl AArch64Vcpu for KvmVcpu {
    fn get_core_reg(&mut self, reg: CoreRegister) -> Result<u64, VcpuError> {
        let mut bytes = [0; 8];
        let len = self.vcpu_fd.get_one_reg(reg.to_kvm_reg(), &mut bytes)?;
        assert_eq!(len, 8);

        let value = u64::from_le_bytes(bytes);
        Ok(value)
    }

    fn set_core_reg(&mut self, reg: CoreRegister, value: u64) -> Result<(), VcpuError> {
        let len = self
            .vcpu_fd
            .set_one_reg(reg.to_kvm_reg(), &value.to_le_bytes())?;
        assert_eq!(len, 8);

        Ok(())
    }

    fn get_sys_reg(&mut self, _reg: SysRegister) -> Result<u64, VcpuError> {
        todo!()
    }

    fn set_sys_reg(&mut self, _reg: SysRegister, _value: u64) -> Result<(), VcpuError> {
        todo!()
    }
}

impl HypervisorVcpu for KvmVcpu {
    fn post_init_within_thread(&mut self) -> Result<(), VcpuError> {
        todo!()
    }

    fn run(&mut self) -> Result<VmExitReason, VcpuError> {
        todo!()
    }
}
