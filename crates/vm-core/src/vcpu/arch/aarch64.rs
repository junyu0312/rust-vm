use crate::vcpu::Vcpu;
use crate::vcpu::arch::aarch64::reg::CoreRegister;
use crate::vcpu::arch::aarch64::reg::SysRegister;
use crate::vcpu::arch::aarch64::reg::cnthctl_el2::CnthctlEl2;
use crate::vcpu::arch::aarch64::reg::sctlr_el1::SctlrEl1;

pub mod reg;

pub trait AArch64Vcpu: Vcpu {
    fn get_core_reg(&self, reg: CoreRegister) -> anyhow::Result<u64>;

    fn set_core_reg(&self, reg: CoreRegister, value: u64) -> anyhow::Result<()>;

    fn get_sys_reg(&self, reg: SysRegister) -> anyhow::Result<u64>;

    fn set_sys_reg(&self, reg: SysRegister, value: u64) -> anyhow::Result<()>;

    fn get_sctlr_el1(&self) -> anyhow::Result<SctlrEl1> {
        Ok(SctlrEl1::from_bits_retain(
            self.get_sys_reg(SysRegister::SctlrEl1)?,
        ))
    }

    fn set_sctlr_el1(&self, sctlr_el1: SctlrEl1) -> anyhow::Result<()> {
        self.set_sys_reg(SysRegister::SctlrEl1, sctlr_el1.bits())?;

        Ok(())
    }

    fn get_cnthctl_el2(&self) -> anyhow::Result<CnthctlEl2> {
        Ok(CnthctlEl2::from_bits_retain(
            self.get_sys_reg(SysRegister::CnthctlEl2)?,
        ))
    }

    fn set_cnthctl_el2(&self, cnthctl_el2: CnthctlEl2) -> anyhow::Result<()> {
        self.set_sys_reg(SysRegister::CnthctlEl2, cnthctl_el2.bits())?;

        Ok(())
    }
}
