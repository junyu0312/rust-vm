use crate::arch::aarch64::AArch64;
use crate::arch::aarch64::vcpu::reg::cnthctl_el2::CnthctlEl2;
use crate::arch::aarch64::vcpu::reg::sctlr_el1::SctlrEl1;
use crate::arch::aarch64::vcpu::reg::*;
use crate::arch::vcpu::Vcpu;
use crate::error::Result;

pub mod reg;

pub trait AArch64Vcpu: Vcpu<AArch64> {
    fn get_core_reg(&self, reg: CoreRegister) -> Result<u64>;

    fn set_core_reg(&self, reg: CoreRegister, value: u64) -> Result<()>;

    fn get_sys_reg(&self, reg: SysRegister) -> Result<u64>;

    fn set_sys_reg(&self, reg: SysRegister, value: u64) -> Result<()>;

    fn get_sctlr_el1(&self) -> Result<SctlrEl1> {
        Ok(SctlrEl1::from_bits_retain(
            self.get_sys_reg(SysRegister::SctlrEl1)?,
        ))
    }

    fn set_sctlr_el1(&self, sctlr_el1: SctlrEl1) -> Result<()> {
        self.set_sys_reg(SysRegister::SctlrEl1, sctlr_el1.bits())
    }

    fn get_cnthctl_el2(&self) -> Result<CnthctlEl2> {
        Ok(CnthctlEl2::from_bits_retain(
            self.get_sys_reg(SysRegister::CnthctlEl2)?,
        ))
    }

    fn set_cnthctl_el2(&self, cnthctl_el2: CnthctlEl2) -> Result<()> {
        self.set_sys_reg(SysRegister::CnthctlEl2, cnthctl_el2.bits())
    }

    fn get_smc_function_id(&self) -> Result<u32> {
        Ok(self.get_core_reg(CoreRegister::X0)? as u32)
    }

    fn get_smc_arg1(&self) -> Result<u64> {
        self.get_core_reg(CoreRegister::X1)
    }

    fn get_smc_arg2(&self) -> Result<u64> {
        self.get_core_reg(CoreRegister::X2)
    }

    fn get_smc_arg3(&self) -> Result<u64> {
        self.get_core_reg(CoreRegister::X3)
    }

    fn set_smc_return_value(&self, x0: u32, x1: u32, x2: u32, x3: u32) -> Result<()> {
        self.set_core_reg(CoreRegister::X0, x0 as u64)?;
        self.set_core_reg(CoreRegister::X1, x1 as u64)?;
        self.set_core_reg(CoreRegister::X2, x2 as u64)?;
        self.set_core_reg(CoreRegister::X3, x3 as u64)?;

        Ok(())
    }
}
