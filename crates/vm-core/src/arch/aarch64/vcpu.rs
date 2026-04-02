use crate::arch::aarch64::vcpu::reg::cnthctl_el2::CnthctlEl2;
use crate::arch::aarch64::vcpu::reg::sctlr_el1::SctlrEl1;
use crate::arch::aarch64::vcpu::reg::*;
use crate::cpu::error::VcpuError;

pub mod reg;

pub fn setup_cpu(
    x0: u64, // dtb for boot cpu, otherwise context_id
    start_pc: u64,
    cpu_id: usize,
    vcpu: &mut dyn AArch64Vcpu,
) -> Result<(), VcpuError> {
    vcpu.set_sys_reg(SysRegister::MpidrEl1, cpu_id as u64)?;

    vcpu.set_core_reg(CoreRegister::X0, x0)?;
    vcpu.set_core_reg(CoreRegister::X1, 0)?;
    vcpu.set_core_reg(CoreRegister::X2, 0)?;
    vcpu.set_core_reg(CoreRegister::X3, 0)?;
    vcpu.set_core_reg(CoreRegister::PC, start_pc)?;

    {
        // CPU mode

        let mut pstate = vcpu.get_core_reg(CoreRegister::PState)?;
        pstate |= 0x03C0; // DAIF
        pstate &= !0xf; // Clear low 4 bits
        pstate |= 0x0005; // El1h
        vcpu.set_core_reg(CoreRegister::PState, pstate)?;

        // more, non secure el1
        if false {
            todo!()
        }
    }

    {
        // Caches, MMUs

        let mut sctlr_el1 = vcpu.get_sctlr_el1()?;
        sctlr_el1.remove(SctlrEl1::M); // Disable MMU
        sctlr_el1.remove(SctlrEl1::I); // Disable I-cache
        vcpu.set_sctlr_el1(sctlr_el1)?;
    }

    {
        // Architected timers

        if false {
            todo!(
                "CNTFRQ must be programmed with the timer frequency and CNTVOFF must be programmed with a consistent value on all CPUs."
            );
        }

        if false {
            // MacOS get panic, should we enable this in Linux?
            let mut cnthctl_el2 = vcpu.get_cnthctl_el2()?;
            cnthctl_el2.insert(CnthctlEl2::EL1PCTEN); // TODO: or bit0?(https://www.kernel.org/doc/html/v5.3/arm64/booting.html)
            vcpu.set_cnthctl_el2(cnthctl_el2)?;
        }
    }

    {
        // Coherency

        // Do nothing
    }

    {
        // System registers

        if false {
            todo!()
        }
    }

    Ok(())
}

pub trait AArch64Vcpu {
    fn get_core_reg(&mut self, reg: CoreRegister) -> Result<u64, VcpuError>;

    fn set_core_reg(&mut self, reg: CoreRegister, value: u64) -> Result<(), VcpuError>;

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
}
