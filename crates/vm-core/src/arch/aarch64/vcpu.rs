use vm_aarch64::register::cnthctl_el2::CnthctlEl2;
use vm_aarch64::register::sctlr_el1::SctlrEl1;

use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::FpRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::arch::registers::aarch64::AArch64CoreRegisters;
use crate::arch::registers::aarch64::AArch64Registers;
use crate::arch::registers::aarch64::AArch64SysRegisters;
use crate::cpu::error::VcpuError;

pub mod reg;

pub trait AArch64Vcpu {
    fn read_registers(&mut self) -> Result<AArch64Registers, VcpuError>;

    fn write_registers(&mut self, registers: AArch64Registers) -> Result<(), VcpuError>;

    fn read_core_registers(&mut self) -> Result<AArch64CoreRegisters, VcpuError>;

    fn write_core_registers(&mut self, registers: AArch64CoreRegisters) -> Result<(), VcpuError>;

    fn read_sys_registers(&mut self) -> Result<AArch64SysRegisters, VcpuError>;

    fn write_sys_registers(&mut self, registers: AArch64SysRegisters) -> Result<(), VcpuError>;

    fn get_core_reg(&mut self, reg: CoreRegister) -> Result<u64, VcpuError>;

    fn set_core_reg(&mut self, reg: CoreRegister, value: u64) -> Result<(), VcpuError>;

    fn get_fp_reg(&mut self, reg: FpRegister) -> Result<u128, VcpuError>;

    fn set_fp_reg(&mut self, reg: FpRegister, value: u128) -> Result<(), VcpuError>;

    fn get_sys_reg(&mut self, reg: SysRegister) -> Result<u64, VcpuError>;

    fn set_sys_reg(&mut self, reg: SysRegister, value: u64) -> Result<(), VcpuError>;

    fn get_sctlr_el1(&mut self) -> Result<SctlrEl1, VcpuError> {
        Ok(SctlrEl1::from_bits_retain(
            self.get_sys_reg(SysRegister::SctlrEl1)?,
        ))
    }

    fn set_sctlr_el1(&mut self, sctlr_el1: SctlrEl1) -> Result<(), VcpuError> {
        self.set_sys_reg(SysRegister::SctlrEl1, sctlr_el1.bits())
    }

    fn get_cnthctl_el2(&mut self) -> Result<CnthctlEl2, VcpuError> {
        Ok(CnthctlEl2::from_bits_retain(
            self.get_sys_reg(SysRegister::CnthctlEl2)?,
        ))
    }

    fn set_cnthctl_el2(&mut self, cnthctl_el2: CnthctlEl2) -> Result<(), VcpuError> {
        self.set_sys_reg(SysRegister::CnthctlEl2, cnthctl_el2.bits())
    }

    fn get_smc_function_id(&mut self) -> Result<u32, VcpuError> {
        Ok(self.get_core_reg(CoreRegister::X0)? as u32)
    }

    fn get_smc_arg1(&mut self) -> Result<u64, VcpuError> {
        self.get_core_reg(CoreRegister::X1)
    }

    fn get_smc_arg2(&mut self) -> Result<u64, VcpuError> {
        self.get_core_reg(CoreRegister::X2)
    }

    fn get_smc_arg3(&mut self) -> Result<u64, VcpuError> {
        self.get_core_reg(CoreRegister::X3)
    }

    fn set_smc_return_value(
        &mut self,
        x0: u32,
        x1: u32,
        x2: u32,
        x3: u32,
    ) -> Result<(), VcpuError> {
        self.set_core_reg(CoreRegister::X0, x0 as u64)?;
        self.set_core_reg(CoreRegister::X1, x1 as u64)?;
        self.set_core_reg(CoreRegister::X2, x2 as u64)?;
        self.set_core_reg(CoreRegister::X3, x3 as u64)?;

        Ok(())
    }

    fn translate_gva_to_gpa(&self, gva: u64) -> Result<u64, VcpuError>;
}
