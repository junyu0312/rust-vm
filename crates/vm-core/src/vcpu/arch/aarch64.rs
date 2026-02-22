use crate::arch::aarch64::AArch64;
use crate::vcpu::Vcpu;
use crate::vcpu::arch::aarch64::reg::CoreRegister;
use crate::vcpu::arch::aarch64::reg::NonDebugSysRegister;
use crate::vcpu::arch::aarch64::reg::SpecialPurposeRegister;
use crate::vcpu::arch::aarch64::reg::cnthctl_el2::CnthctlEl2;
use crate::vcpu::arch::aarch64::reg::sctlr_el1::SctlrEl1;

pub mod reg;

pub trait AArch64Vcpu: Vcpu<AArch64> {
    fn get_core_reg(&self, reg: CoreRegister) -> anyhow::Result<u64>;

    fn set_core_reg(&self, reg: CoreRegister, value: u64) -> anyhow::Result<()>;

    fn get_non_debug_sys_reg(&self, reg: NonDebugSysRegister) -> anyhow::Result<u64>;

    fn set_non_debug_sys_reg(&self, reg: NonDebugSysRegister, value: u64) -> anyhow::Result<()>;

    fn get_special_purpose_reg(&self, reg: SpecialPurposeRegister) -> anyhow::Result<u64>;

    fn set_special_purpose_reg(
        &self,
        reg: SpecialPurposeRegister,
        value: u64,
    ) -> anyhow::Result<u64>;

    fn get_sctlr_el1(&self) -> anyhow::Result<SctlrEl1> {
        Ok(SctlrEl1::from_bits_retain(
            self.get_non_debug_sys_reg(NonDebugSysRegister::SctlrEl1)?,
        ))
    }

    fn set_sctlr_el1(&self, sctlr_el1: SctlrEl1) -> anyhow::Result<()> {
        self.set_non_debug_sys_reg(NonDebugSysRegister::SctlrEl1, sctlr_el1.bits())?;

        Ok(())
    }

    fn get_cnthctl_el2(&self) -> anyhow::Result<CnthctlEl2> {
        Ok(CnthctlEl2::from_bits_retain(
            self.get_non_debug_sys_reg(NonDebugSysRegister::CnthctlEl2)?,
        ))
    }

    fn set_cnthctl_el2(&self, cnthctl_el2: CnthctlEl2) -> anyhow::Result<()> {
        self.set_non_debug_sys_reg(NonDebugSysRegister::CnthctlEl2, cnthctl_el2.bits())?;

        Ok(())
    }

    fn get_icc_pmr_el1(&self) -> anyhow::Result<u64> {
        self.get_special_purpose_reg(SpecialPurposeRegister::ICC_PMR_EL1)
    }
}
