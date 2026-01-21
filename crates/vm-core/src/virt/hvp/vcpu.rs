use applevisor_sys::hv_exit_reason_t;
use applevisor_sys::hv_reg_t;
use applevisor_sys::hv_sys_reg_t;
use tracing::trace;

use crate::device::pio::IoAddressSpace;
use crate::vcpu::Vcpu;
use crate::vcpu::arch::aarch64::AArch64Vcpu;
use crate::vcpu::arch::aarch64::reg::CoreRegister;
use crate::vcpu::arch::aarch64::reg::SysRegister;

impl CoreRegister {
    fn to_hvp_reg(&self) -> hv_reg_t {
        match self {
            CoreRegister::X0 => hv_reg_t::X0,
            CoreRegister::X1 => hv_reg_t::X1,
            CoreRegister::X2 => hv_reg_t::X2,
            CoreRegister::X3 => hv_reg_t::X3,
            CoreRegister::X4 => hv_reg_t::X4,
            CoreRegister::X5 => hv_reg_t::X5,
            CoreRegister::X6 => hv_reg_t::X6,
            CoreRegister::X7 => hv_reg_t::X7,
            CoreRegister::X8 => hv_reg_t::X8,
            CoreRegister::X9 => hv_reg_t::X9,
            CoreRegister::X10 => hv_reg_t::X10,
            CoreRegister::X11 => hv_reg_t::X11,
            CoreRegister::X12 => hv_reg_t::X12,
            CoreRegister::X13 => hv_reg_t::X13,
            CoreRegister::X14 => hv_reg_t::X14,
            CoreRegister::X15 => hv_reg_t::X15,
            CoreRegister::X16 => hv_reg_t::X16,
            CoreRegister::X17 => hv_reg_t::X17,
            CoreRegister::X18 => hv_reg_t::X18,
            CoreRegister::X19 => hv_reg_t::X19,
            CoreRegister::X20 => hv_reg_t::X20,
            CoreRegister::X21 => hv_reg_t::X21,
            CoreRegister::X22 => hv_reg_t::X22,
            CoreRegister::X23 => hv_reg_t::X23,
            CoreRegister::X24 => hv_reg_t::X24,
            CoreRegister::X25 => hv_reg_t::X25,
            CoreRegister::X26 => hv_reg_t::X26,
            CoreRegister::X27 => hv_reg_t::X27,
            CoreRegister::X28 => hv_reg_t::X28,
            CoreRegister::X29 => hv_reg_t::X29,
            CoreRegister::X30 => hv_reg_t::X30,
            CoreRegister::PState => hv_reg_t::CPSR,
        }
    }
}

impl SysRegister {
    fn to_hvp_reg(&self) -> hv_sys_reg_t {
        match self {
            SysRegister::SctlrEl1 => hv_sys_reg_t::SCTLR_EL1,
            SysRegister::CnthctlEl2 => hv_sys_reg_t::CNTHCTL_EL2,
        }
    }
}

pub struct HvpVcpu {
    vcpu_id: u64,
    vcpu: applevisor::vcpu::Vcpu,
}

impl HvpVcpu {
    pub fn new(vcpu_id: u64, vcpu: applevisor::vcpu::Vcpu) -> Self {
        HvpVcpu { vcpu_id, vcpu }
    }
}

impl AArch64Vcpu for HvpVcpu {
    fn get_one_reg(&self, reg: CoreRegister) -> anyhow::Result<u64> {
        Ok(self.vcpu.get_reg(reg.to_hvp_reg())?)
    }

    fn set_one_reg(&self, reg: CoreRegister, value: u64) -> anyhow::Result<()> {
        self.vcpu.set_reg(reg.to_hvp_reg(), value)?;

        Ok(())
    }

    fn get_sys_reg(&self, reg: SysRegister) -> anyhow::Result<u64> {
        Ok(self.vcpu.get_sys_reg(reg.to_hvp_reg())?)
    }

    fn set_sys_reg(&self, reg: SysRegister, value: u64) -> anyhow::Result<()> {
        self.vcpu.set_sys_reg(reg.to_hvp_reg(), value)?;

        Ok(())
    }
}

impl Vcpu for HvpVcpu {
    fn run(&mut self, _device: &mut IoAddressSpace) -> anyhow::Result<()> {
        loop {
            self.vcpu.run()?;

            let exit_info = self.vcpu.get_exit_info();

            trace!(self.vcpu_id, ?exit_info, "vm exit");

            match exit_info.reason {
                hv_exit_reason_t::CANCELED => todo!(),
                hv_exit_reason_t::EXCEPTION => todo!(),
                hv_exit_reason_t::VTIMER_ACTIVATED => todo!(),
                hv_exit_reason_t::UNKNOWN => todo!(),
            }
        }
    }
}
