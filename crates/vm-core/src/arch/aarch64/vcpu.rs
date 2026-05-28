use vm_aarch64::register::cnthctl_el2::CnthctlEl2;
use vm_aarch64::register::id_aa64mmfr0_el1::IdAa64mmfr0El1;
use vm_aarch64::register::sctlr_el1::SctlrEl1;
use vm_aarch64::register::tcr_el1::TcrEl1;
use vm_aarch64::register::ttbr1_el1::Ttbr1El1;
use vm_mm::manager::MemoryAddressSpace;

use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::FpRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::arch::mmu::aarch64::translate_gva_to_gpa;
use crate::arch::registers::aarch64::AArch64CoreRegisters;
use crate::arch::registers::aarch64::AArch64Registers;
use crate::arch::registers::aarch64::AArch64SysRegisters;
use crate::virtualization::vcpu::error::VcpuError;

pub mod reg;

pub trait AArch64Vcpu {
    fn get_core_reg(&self, reg: CoreRegister) -> Result<u64, VcpuError>;

    fn set_core_reg(&mut self, reg: CoreRegister, value: u64) -> Result<(), VcpuError>;

    fn get_fp_reg(&self, reg: FpRegister) -> Result<u128, VcpuError>;

    fn set_fp_reg(&mut self, reg: FpRegister, value: u128) -> Result<(), VcpuError>;

    fn get_sys_reg(&self, reg: SysRegister) -> Result<u64, VcpuError>;

    fn set_sys_reg(&mut self, reg: SysRegister, value: u64) -> Result<(), VcpuError>;

    fn read_registers(&self) -> Result<AArch64Registers, VcpuError> {
        Ok(AArch64Registers {
            core: self.read_core_registers()?,
            sys: self.read_sys_registers()?,
        })
    }

    fn write_registers(&mut self, registers: AArch64Registers) -> Result<(), VcpuError> {
        self.write_core_registers(registers.core)?;
        self.write_sys_registers(registers.sys)?;

        Ok(())
    }

    fn read_sys_registers(&self) -> Result<AArch64SysRegisters, VcpuError> {
        Ok(AArch64SysRegisters {
            mpidr_el1: self.get_sys_reg(SysRegister::MpidrEl1)?,
            sctlr_el1: self.get_sys_reg(SysRegister::SctlrEl1)?,
            cnthctl_el2: self.get_sys_reg(SysRegister::CnthctlEl2)?,
        })
    }

    fn write_sys_registers(&mut self, registers: AArch64SysRegisters) -> Result<(), VcpuError> {
        self.set_sys_reg(SysRegister::MpidrEl1, registers.mpidr_el1)?;
        self.set_sys_reg(SysRegister::SctlrEl1, registers.sctlr_el1)?;
        self.set_sys_reg(SysRegister::CnthctlEl2, registers.cnthctl_el2)?;

        Ok(())
    }

    fn read_core_registers(&self) -> Result<AArch64CoreRegisters, VcpuError> {
        let mut general_purpose = [0; 31];
        for (i, gp) in general_purpose.iter_mut().enumerate() {
            *gp = self.get_core_reg(CoreRegister::from_srt(i as u64))?;
        }

        let mut fp = [0; 32];
        for (i, fp) in fp.iter_mut().enumerate() {
            *fp = self.get_fp_reg(FpRegister::from_repr(i).unwrap())?;
        }

        Ok(AArch64CoreRegisters {
            general_purpose,
            sp: self.get_core_reg(CoreRegister::SP)?,
            pc: self.get_core_reg(CoreRegister::PC)?,
            pstate: self.get_core_reg(CoreRegister::PState)?,
            fp,
            fpcr: self.get_core_reg(CoreRegister::Fpcr)?,
            fpsr: self.get_core_reg(CoreRegister::Fpsr)?,
        })
    }

    fn write_core_registers(&mut self, registers: AArch64CoreRegisters) -> Result<(), VcpuError> {
        for gp in 0usize..31 {
            self.set_core_reg(
                CoreRegister::from_srt(gp as u64),
                registers.general_purpose[gp],
            )?;
        }
        self.set_core_reg(CoreRegister::SP, registers.sp)?;
        self.set_core_reg(CoreRegister::PC, registers.pc)?;
        self.set_core_reg(CoreRegister::PState, registers.pstate)?;
        for fp in 0usize..32 {
            self.set_fp_reg(FpRegister::from_repr(fp).unwrap(), registers.fp[fp])?;
        }
        self.set_core_reg(CoreRegister::Fpcr, registers.fpcr)?;
        self.set_core_reg(CoreRegister::Fpsr, registers.fpsr)?;

        Ok(())
    }

    fn get_sctlr_el1(&self) -> Result<SctlrEl1, VcpuError> {
        Ok(SctlrEl1::from_bits_retain(
            self.get_sys_reg(SysRegister::SctlrEl1)?,
        ))
    }

    fn set_sctlr_el1(&mut self, sctlr_el1: SctlrEl1) -> Result<(), VcpuError> {
        self.set_sys_reg(SysRegister::SctlrEl1, sctlr_el1.bits())
    }

    fn get_cnthctl_el2(&self) -> Result<CnthctlEl2, VcpuError> {
        Ok(CnthctlEl2::from_bits_retain(
            self.get_sys_reg(SysRegister::CnthctlEl2)?,
        ))
    }

    fn set_cnthctl_el2(&mut self, cnthctl_el2: CnthctlEl2) -> Result<(), VcpuError> {
        self.set_sys_reg(SysRegister::CnthctlEl2, cnthctl_el2.bits())
    }

    fn get_smc_function_id(&self) -> Result<u32, VcpuError> {
        Ok(self.get_core_reg(CoreRegister::X0)? as u32)
    }

    fn get_smc_arg1(&self) -> Result<u64, VcpuError> {
        self.get_core_reg(CoreRegister::X1)
    }

    fn get_smc_arg2(&self) -> Result<u64, VcpuError> {
        self.get_core_reg(CoreRegister::X2)
    }

    fn get_smc_arg3(&self) -> Result<u64, VcpuError> {
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

    fn mm(&self) -> &MemoryAddressSpace;

    fn translate_gva_to_gpa(&self, gva: u64) -> Result<Option<u64>, VcpuError> {
        let tcr_el1 = || Ok(TcrEl1::from(self.get_sys_reg(SysRegister::TcrEl1)?));
        let ttbr1_el1 = || Ok(Ttbr1El1::from(self.get_sys_reg(SysRegister::Ttbr1El1)?));
        let id_aa64mmfr0_el1 = || {
            Ok(IdAa64mmfr0El1::from(
                self.get_sys_reg(SysRegister::IdAa64mmfr0El1)?,
            ))
        };
        translate_gva_to_gpa(self.mm(), tcr_el1, ttbr1_el1, id_aa64mmfr0_el1, gva)
    }
}
