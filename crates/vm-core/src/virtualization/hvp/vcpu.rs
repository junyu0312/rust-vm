use std::ptr::null_mut;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;

use applevisor_sys::hv_error_t;
use applevisor_sys::hv_vcpu_create;
use applevisor_sys::hv_vcpu_exit_t;
use applevisor_sys::hv_vcpu_get_reg;
use applevisor_sys::hv_vcpu_get_simd_fp_reg;
use applevisor_sys::hv_vcpu_get_sys_reg;
use applevisor_sys::hv_vcpu_run;
use applevisor_sys::hv_vcpu_set_reg;
use applevisor_sys::hv_vcpu_set_simd_fp_reg;
use applevisor_sys::hv_vcpu_set_sys_reg;
use applevisor_sys::hv_vcpu_t;
use applevisor_sys::hv_vcpus_exit;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::WeakSender;
use tokio::sync::mpsc::error::TryRecvError;
use tracing::error;
use vm_aarch64::register::id_aa64mmfr0_el1::IdAa64mmfr0El1;
use vm_aarch64::register::tcr_el1::TcrEl1;
use vm_aarch64::register::ttbr1_el1::Ttbr1El1;
use vm_mm::manager::MemoryAddressSpace;

use crate::arch::aarch64::vcpu::AArch64Vcpu;
use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::FpRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::arch::aarch64::vm_exit::HandleVmExitResult;
use crate::arch::aarch64::vm_exit::handle_vm_exit;
use crate::arch::mmu::aarch64::translate_gva_to_gpa;
use crate::arch::registers::aarch64::AArch64CoreRegisters;
use crate::arch::registers::aarch64::AArch64Registers;
use crate::arch::registers::aarch64::AArch64SysRegisters;
use crate::cpu::vm_exit::VmExit;
use crate::virtualization::hvp::hv_unsafe_call;
use crate::virtualization::hvp::vcpu::register::HvpReg;
use crate::virtualization::hvp::vcpu::vm_exit::to_vm_exit;
use crate::virtualization::vcpu::HypervisorVcpu;
use crate::virtualization::vcpu::command::VcpuCommand;
use crate::virtualization::vcpu::command::VcpuCommandRequest;
use crate::virtualization::vcpu::command::VcpuCommandResponse;
use crate::virtualization::vcpu::error::VcpuError;

mod register;
mod vm_exit;

struct HvpVcpuInternal {
    /// handler for apple hypervisor vcpu
    vcpu: hv_vcpu_t,
    mm: Arc<MemoryAddressSpace>,
}

impl AArch64Vcpu for HvpVcpuInternal {
    fn read_registers(&mut self) -> Result<AArch64Registers, VcpuError> {
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

    fn read_core_registers(&mut self) -> Result<AArch64CoreRegisters, VcpuError> {
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

    fn read_sys_registers(&mut self) -> Result<AArch64SysRegisters, VcpuError> {
        Ok(AArch64SysRegisters {
            dbgbvr0_el1: self.get_sys_reg(SysRegister::Dbgbvr0El1)?,
            dbgbcr0_el1: self.get_sys_reg(SysRegister::Dbgbcr0El1)?,
            dbgwvr0_el1: self.get_sys_reg(SysRegister::Dbgwvr0El1)?,
            dbgwcr0_el1: self.get_sys_reg(SysRegister::Dbgwcr0El1)?,
            dbgbvr1_el1: self.get_sys_reg(SysRegister::Dbgbvr1El1)?,
            dbgbcr1_el1: self.get_sys_reg(SysRegister::Dbgbcr1El1)?,
            dbgwvr1_el1: self.get_sys_reg(SysRegister::Dbgwvr1El1)?,
            dbgwcr1_el1: self.get_sys_reg(SysRegister::Dbgwcr1El1)?,
            mdccint_el1: self.get_sys_reg(SysRegister::MdccintEl1)?,
            mdscr_el1: self.get_sys_reg(SysRegister::MdscrEl1)?,
            dbgbvr2_el1: self.get_sys_reg(SysRegister::Dbgbvr2El1)?,
            dbgbcr2_el1: self.get_sys_reg(SysRegister::Dbgbcr2El1)?,
            dbgwvr2_el1: self.get_sys_reg(SysRegister::Dbgwvr2El1)?,
            dbgwcr2_el1: self.get_sys_reg(SysRegister::Dbgwcr2El1)?,
            dbgbvr3_el1: self.get_sys_reg(SysRegister::Dbgbvr3El1)?,
            dbgbcr3_el1: self.get_sys_reg(SysRegister::Dbgbcr3El1)?,
            dbgwvr3_el1: self.get_sys_reg(SysRegister::Dbgwvr3El1)?,
            dbgwcr3_el1: self.get_sys_reg(SysRegister::Dbgwcr3El1)?,
            dbgbvr4_el1: self.get_sys_reg(SysRegister::Dbgbvr4El1)?,
            dbgbcr4_el1: self.get_sys_reg(SysRegister::Dbgbcr4El1)?,
            dbgwvr4_el1: self.get_sys_reg(SysRegister::Dbgwvr4El1)?,
            dbgwcr4_el1: self.get_sys_reg(SysRegister::Dbgwcr4El1)?,
            dbgbvr5_el1: self.get_sys_reg(SysRegister::Dbgbvr5El1)?,
            dbgbcr5_el1: self.get_sys_reg(SysRegister::Dbgbcr5El1)?,
            dbgwvr5_el1: self.get_sys_reg(SysRegister::Dbgwvr5El1)?,
            dbgwcr5_el1: self.get_sys_reg(SysRegister::Dbgwcr5El1)?,
            dbgbvr6_el1: self.get_sys_reg(SysRegister::Dbgbvr6El1)?,
            dbgbcr6_el1: self.get_sys_reg(SysRegister::Dbgbcr6El1)?,
            dbgwvr6_el1: self.get_sys_reg(SysRegister::Dbgwvr6El1)?,
            dbgwcr6_el1: self.get_sys_reg(SysRegister::Dbgwcr6El1)?,
            dbgbvr7_el1: self.get_sys_reg(SysRegister::Dbgbvr7El1)?,
            dbgbcr7_el1: self.get_sys_reg(SysRegister::Dbgbcr7El1)?,
            dbgwvr7_el1: self.get_sys_reg(SysRegister::Dbgwvr7El1)?,
            dbgwcr7_el1: self.get_sys_reg(SysRegister::Dbgwcr7El1)?,
            dbgbvr8_el1: self.get_sys_reg(SysRegister::Dbgbvr8El1)?,
            dbgbcr8_el1: self.get_sys_reg(SysRegister::Dbgbcr8El1)?,
            dbgwvr8_el1: self.get_sys_reg(SysRegister::Dbgwvr8El1)?,
            dbgwcr8_el1: self.get_sys_reg(SysRegister::Dbgwcr8El1)?,
            dbgbvr9_el1: self.get_sys_reg(SysRegister::Dbgbvr9El1)?,
            dbgbcr9_el1: self.get_sys_reg(SysRegister::Dbgbcr9El1)?,
            dbgwvr9_el1: self.get_sys_reg(SysRegister::Dbgwvr9El1)?,
            dbgwcr9_el1: self.get_sys_reg(SysRegister::Dbgwcr9El1)?,
            dbgbvr10_el1: self.get_sys_reg(SysRegister::Dbgbvr10El1)?,
            dbgbcr10_el1: self.get_sys_reg(SysRegister::Dbgbcr10El1)?,
            dbgwvr10_el1: self.get_sys_reg(SysRegister::Dbgwvr10El1)?,
            dbgwcr10_el1: self.get_sys_reg(SysRegister::Dbgwcr10El1)?,
            dbgbvr11_el1: self.get_sys_reg(SysRegister::Dbgbvr11El1)?,
            dbgbcr11_el1: self.get_sys_reg(SysRegister::Dbgbcr11El1)?,
            dbgwvr11_el1: self.get_sys_reg(SysRegister::Dbgwvr11El1)?,
            dbgwcr11_el1: self.get_sys_reg(SysRegister::Dbgwcr11El1)?,
            dbgbvr12_el1: self.get_sys_reg(SysRegister::Dbgbvr12El1)?,
            dbgbcr12_el1: self.get_sys_reg(SysRegister::Dbgbcr12El1)?,
            dbgwvr12_el1: self.get_sys_reg(SysRegister::Dbgwvr12El1)?,
            dbgwcr12_el1: self.get_sys_reg(SysRegister::Dbgwcr12El1)?,
            dbgbvr13_el1: self.get_sys_reg(SysRegister::Dbgbvr13El1)?,
            dbgbcr13_el1: self.get_sys_reg(SysRegister::Dbgbcr13El1)?,
            dbgwvr13_el1: self.get_sys_reg(SysRegister::Dbgwvr13El1)?,
            dbgwcr13_el1: self.get_sys_reg(SysRegister::Dbgwcr13El1)?,
            dbgbvr14_el1: self.get_sys_reg(SysRegister::Dbgbvr14El1)?,
            dbgbcr14_el1: self.get_sys_reg(SysRegister::Dbgbcr14El1)?,
            dbgwvr14_el1: self.get_sys_reg(SysRegister::Dbgwvr14El1)?,
            dbgwcr14_el1: self.get_sys_reg(SysRegister::Dbgwcr14El1)?,
            dbgbvr15_el1: self.get_sys_reg(SysRegister::Dbgbvr15El1)?,
            dbgbcr15_el1: self.get_sys_reg(SysRegister::Dbgbcr15El1)?,
            dbgwvr15_el1: self.get_sys_reg(SysRegister::Dbgwvr15El1)?,
            dbgwcr15_el1: self.get_sys_reg(SysRegister::Dbgwcr15El1)?,
            midr_el1: self.get_sys_reg(SysRegister::MidrEl1)?,
            mpidr_el1: self.get_sys_reg(SysRegister::MpidrEl1)?,
            id_aa64pfr0_el1: self.get_sys_reg(SysRegister::IdAa64pfr0El1)?,
            id_aa64pfr1_el1: self.get_sys_reg(SysRegister::IdAa64pfr1El1)?,
            id_aa64zfr0_el1: self.get_sys_reg(SysRegister::IdAa64zfr0El1)?,
            id_aa64smfr0_el1: self.get_sys_reg(SysRegister::IdAa64smfr0El1)?,
            id_aa64dfr0_el1: self.get_sys_reg(SysRegister::IdAa64dfr0El1)?,
            id_aa64dfr1_el1: self.get_sys_reg(SysRegister::IdAa64dfr1El1)?,
            id_aa64isar0_el1: self.get_sys_reg(SysRegister::IdAa64isar0El1)?,
            id_aa64isar1_el1: self.get_sys_reg(SysRegister::IdAa64isar1El1)?,
            id_aa64mmfr0_el1: self.get_sys_reg(SysRegister::IdAa64mmfr0El1)?,
            id_aa64mmfr1_el1: self.get_sys_reg(SysRegister::IdAa64mmfr1El1)?,
            id_aa64mmfr2_el1: self.get_sys_reg(SysRegister::IdAa64mmfr2El1)?,
            sctlr_el1: self.get_sys_reg(SysRegister::SctlrEl1)?,
            cpacr_el1: self.get_sys_reg(SysRegister::CpacrEl1)?,
            actlr_el1: self.get_sys_reg(SysRegister::ActlrEl1)?,
            smpri_el1: self.get_sys_reg(SysRegister::SmpriEl1)?,
            smcr_el1: self.get_sys_reg(SysRegister::SmcrEl1)?,
            ttbr0_el1: self.get_sys_reg(SysRegister::Ttbr0El1)?,
            ttbr1_el1: self.get_sys_reg(SysRegister::Ttbr1El1)?,
            tcr_el1: self.get_sys_reg(SysRegister::TcrEl1)?,
            apiakeylo_el1: self.get_sys_reg(SysRegister::ApiakeyloEl1)?,
            apiakeyhi_el1: self.get_sys_reg(SysRegister::ApiakeyhiEl1)?,
            apibkeylo_el1: self.get_sys_reg(SysRegister::ApibkeyloEl1)?,
            apibkeyhi_el1: self.get_sys_reg(SysRegister::ApibkeyhiEl1)?,
            apdakeylo_el1: self.get_sys_reg(SysRegister::ApdakeyloEl1)?,
            apdakeyhi_el1: self.get_sys_reg(SysRegister::ApdakeyhiEl1)?,
            apdbkeylo_el1: self.get_sys_reg(SysRegister::ApdbkeyloEl1)?,
            apdbkeyhi_el1: self.get_sys_reg(SysRegister::ApdbkeyhiEl1)?,
            apgakeylo_el1: self.get_sys_reg(SysRegister::ApgakeyloEl1)?,
            apgakeyhi_el1: self.get_sys_reg(SysRegister::ApgakeyhiEl1)?,
            spsr_el1: self.get_sys_reg(SysRegister::SpsrEl1)?,
            elr_el1: self.get_sys_reg(SysRegister::ElrEl1)?,
            sp_el0: self.get_sys_reg(SysRegister::SpEl0)?,
            afsr0_el1: self.get_sys_reg(SysRegister::Afsr0El1)?,
            afsr1_el1: self.get_sys_reg(SysRegister::Afsr1El1)?,
            esr_el1: self.get_sys_reg(SysRegister::EsrEl1)?,
            far_el1: self.get_sys_reg(SysRegister::FarEl1)?,
            par_el1: self.get_sys_reg(SysRegister::ParEl1)?,
            mair_el1: self.get_sys_reg(SysRegister::MairEl1)?,
            amair_el1: self.get_sys_reg(SysRegister::AmairEl1)?,
            vbar_el1: self.get_sys_reg(SysRegister::VbarEl1)?,
            contextidr_el1: self.get_sys_reg(SysRegister::ContextidrEl1)?,
            tpidr_el1: self.get_sys_reg(SysRegister::TpidrEl1)?,
            scxtnum_el1: self.get_sys_reg(SysRegister::ScxtnumEl1)?,
            cntkctl_el1: self.get_sys_reg(SysRegister::CntkctlEl1)?,
            csselr_el1: self.get_sys_reg(SysRegister::CsselrEl1)?,
            tpidr_el0: self.get_sys_reg(SysRegister::TpidrEl0)?,
            tpidrro_el0: self.get_sys_reg(SysRegister::TpidrroEl0)?,
            tpidr2_el0: self.get_sys_reg(SysRegister::Tpidr2El0)?,
            scxtnum_el0: self.get_sys_reg(SysRegister::ScxtnumEl0)?,
            cntv_ctl_el0: self.get_sys_reg(SysRegister::CntvCtlEl0)?,
            cntv_cval_el0: self.get_sys_reg(SysRegister::CntvCvalEl0)?,
            sp_el1: self.get_sys_reg(SysRegister::SpEl1)?,
            cntp_ctl_el0: self.get_sys_reg(SysRegister::CntpCtlEl0)?,
            cntp_cval_el0: self.get_sys_reg(SysRegister::CntpCvalEl0)?,
            cntp_tval_el0: self.get_sys_reg(SysRegister::CntpTvalEl0)?,
            cnthctl_el2: self.get_sys_reg(SysRegister::CnthctlEl2)?,
            cnthp_ctl_el2: self.get_sys_reg(SysRegister::CnthpCtlEl2)?,
            cnthp_cval_el2: self.get_sys_reg(SysRegister::CnthpCvalEl2)?,
            cnthp_tval_el2: self.get_sys_reg(SysRegister::CnthpTvalEl2)?,
            cntvoff_el2: self.get_sys_reg(SysRegister::CntvoffEl2)?,
            cptr_el2: self.get_sys_reg(SysRegister::CptrEl2)?,
            elr_el2: self.get_sys_reg(SysRegister::ElrEl2)?,
            esr_el2: self.get_sys_reg(SysRegister::EsrEl2)?,
            far_el2: self.get_sys_reg(SysRegister::FarEl2)?,
            hcr_el2: self.get_sys_reg(SysRegister::HcrEl2)?,
            hpfar_el2: self.get_sys_reg(SysRegister::HpfarEl2)?,
            mair_el2: self.get_sys_reg(SysRegister::MairEl2)?,
            // mdcr_el2: self
            //     .get_sys_reg(SysRegister::MdcrEl2)
            //     ?,
            sctlr_el2: self.get_sys_reg(SysRegister::SctlrEl2)?,
            spsr_el2: self.get_sys_reg(SysRegister::SpsrEl2)?,
            sp_el2: self.get_sys_reg(SysRegister::SpEl2)?,
            tcr_el2: self.get_sys_reg(SysRegister::TcrEl2)?,
            tpidr_el2: self.get_sys_reg(SysRegister::TpidrEl2)?,
            ttbr0_el2: self.get_sys_reg(SysRegister::Ttbr0El2)?,
            ttbr1_el2: self.get_sys_reg(SysRegister::Ttbr1El2)?,
            vbar_el2: self.get_sys_reg(SysRegister::VbarEl2)?,
            vmpidr_el2: self.get_sys_reg(SysRegister::VmpidrEl2)?,
            vpidr_el2: self.get_sys_reg(SysRegister::VpidrEl2)?,
            vtcr_el2: self.get_sys_reg(SysRegister::VtcrEl2)?,
            vttbr_el2: self.get_sys_reg(SysRegister::VttbrEl2)?,
        })
    }

    fn write_sys_registers(&mut self, registers: AArch64SysRegisters) -> Result<(), VcpuError> {
        self.set_sys_reg(SysRegister::Dbgbvr0El1, registers.dbgbvr0_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr0El1, registers.dbgbcr0_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr0El1, registers.dbgwvr0_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr0El1, registers.dbgwcr0_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr1El1, registers.dbgbvr1_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr1El1, registers.dbgbcr1_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr1El1, registers.dbgwvr1_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr1El1, registers.dbgwcr1_el1)?;
        self.set_sys_reg(SysRegister::MdccintEl1, registers.mdccint_el1)?;
        self.set_sys_reg(SysRegister::MdscrEl1, registers.mdscr_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr2El1, registers.dbgbvr2_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr2El1, registers.dbgbcr2_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr2El1, registers.dbgwvr2_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr2El1, registers.dbgwcr2_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr3El1, registers.dbgbvr3_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr3El1, registers.dbgbcr3_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr3El1, registers.dbgwvr3_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr3El1, registers.dbgwcr3_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr4El1, registers.dbgbvr4_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr4El1, registers.dbgbcr4_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr4El1, registers.dbgwvr4_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr4El1, registers.dbgwcr4_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr5El1, registers.dbgbvr5_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr5El1, registers.dbgbcr5_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr5El1, registers.dbgwvr5_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr5El1, registers.dbgwcr5_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr6El1, registers.dbgbvr6_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr6El1, registers.dbgbcr6_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr6El1, registers.dbgwvr6_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr6El1, registers.dbgwcr6_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr7El1, registers.dbgbvr7_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr7El1, registers.dbgbcr7_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr7El1, registers.dbgwvr7_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr7El1, registers.dbgwcr7_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr8El1, registers.dbgbvr8_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr8El1, registers.dbgbcr8_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr8El1, registers.dbgwvr8_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr8El1, registers.dbgwcr8_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr9El1, registers.dbgbvr9_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr9El1, registers.dbgbcr9_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr9El1, registers.dbgwvr9_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr9El1, registers.dbgwcr9_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr10El1, registers.dbgbvr10_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr10El1, registers.dbgbcr10_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr10El1, registers.dbgwvr10_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr10El1, registers.dbgwcr10_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr11El1, registers.dbgbvr11_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr11El1, registers.dbgbcr11_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr11El1, registers.dbgwvr11_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr11El1, registers.dbgwcr11_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr12El1, registers.dbgbvr12_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr12El1, registers.dbgbcr12_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr12El1, registers.dbgwvr12_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr12El1, registers.dbgwcr12_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr13El1, registers.dbgbvr13_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr13El1, registers.dbgbcr13_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr13El1, registers.dbgwvr13_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr13El1, registers.dbgwcr13_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr14El1, registers.dbgbvr14_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr14El1, registers.dbgbcr14_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr14El1, registers.dbgwvr14_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr14El1, registers.dbgwcr14_el1)?;
        self.set_sys_reg(SysRegister::Dbgbvr15El1, registers.dbgbvr15_el1)?;
        self.set_sys_reg(SysRegister::Dbgbcr15El1, registers.dbgbcr15_el1)?;
        self.set_sys_reg(SysRegister::Dbgwvr15El1, registers.dbgwvr15_el1)?;
        self.set_sys_reg(SysRegister::Dbgwcr15El1, registers.dbgwcr15_el1)?;
        self.set_sys_reg(SysRegister::MidrEl1, registers.midr_el1)?;
        self.set_sys_reg(SysRegister::MpidrEl1, registers.mpidr_el1)?;
        self.set_sys_reg(SysRegister::IdAa64pfr0El1, registers.id_aa64pfr0_el1)?;
        self.set_sys_reg(SysRegister::IdAa64pfr1El1, registers.id_aa64pfr1_el1)?;
        self.set_sys_reg(SysRegister::IdAa64zfr0El1, registers.id_aa64zfr0_el1)?;
        self.set_sys_reg(SysRegister::IdAa64smfr0El1, registers.id_aa64smfr0_el1)?;
        self.set_sys_reg(SysRegister::IdAa64dfr0El1, registers.id_aa64dfr0_el1)?;
        self.set_sys_reg(SysRegister::IdAa64dfr1El1, registers.id_aa64dfr1_el1)?;
        self.set_sys_reg(SysRegister::IdAa64isar0El1, registers.id_aa64isar0_el1)?;
        self.set_sys_reg(SysRegister::IdAa64isar1El1, registers.id_aa64isar1_el1)?;
        self.set_sys_reg(SysRegister::IdAa64mmfr0El1, registers.id_aa64mmfr0_el1)?;
        self.set_sys_reg(SysRegister::IdAa64mmfr1El1, registers.id_aa64mmfr1_el1)?;
        self.set_sys_reg(SysRegister::IdAa64mmfr2El1, registers.id_aa64mmfr2_el1)?;
        self.set_sys_reg(SysRegister::SctlrEl1, registers.sctlr_el1)?;
        self.set_sys_reg(SysRegister::CpacrEl1, registers.cpacr_el1)?;
        self.set_sys_reg(SysRegister::ActlrEl1, registers.actlr_el1)?;
        self.set_sys_reg(SysRegister::SmpriEl1, registers.smpri_el1)?;
        self.set_sys_reg(SysRegister::SmcrEl1, registers.smcr_el1)?;
        self.set_sys_reg(SysRegister::Ttbr0El1, registers.ttbr0_el1)?;
        self.set_sys_reg(SysRegister::Ttbr1El1, registers.ttbr1_el1)?;
        self.set_sys_reg(SysRegister::TcrEl1, registers.tcr_el1)?;
        self.set_sys_reg(SysRegister::ApiakeyloEl1, registers.apiakeylo_el1)?;
        self.set_sys_reg(SysRegister::ApiakeyhiEl1, registers.apiakeyhi_el1)?;
        self.set_sys_reg(SysRegister::ApibkeyloEl1, registers.apibkeylo_el1)?;
        self.set_sys_reg(SysRegister::ApibkeyhiEl1, registers.apibkeyhi_el1)?;
        self.set_sys_reg(SysRegister::ApdakeyloEl1, registers.apdakeylo_el1)?;
        self.set_sys_reg(SysRegister::ApdakeyhiEl1, registers.apdakeyhi_el1)?;
        self.set_sys_reg(SysRegister::ApdbkeyloEl1, registers.apdbkeylo_el1)?;
        self.set_sys_reg(SysRegister::ApdbkeyhiEl1, registers.apdbkeyhi_el1)?;
        self.set_sys_reg(SysRegister::ApgakeyloEl1, registers.apgakeylo_el1)?;
        self.set_sys_reg(SysRegister::ApgakeyhiEl1, registers.apgakeyhi_el1)?;
        self.set_sys_reg(SysRegister::SpsrEl1, registers.spsr_el1)?;
        self.set_sys_reg(SysRegister::ElrEl1, registers.elr_el1)?;
        self.set_sys_reg(SysRegister::SpEl0, registers.sp_el0)?;
        self.set_sys_reg(SysRegister::Afsr0El1, registers.afsr0_el1)?;
        self.set_sys_reg(SysRegister::Afsr1El1, registers.afsr1_el1)?;
        self.set_sys_reg(SysRegister::EsrEl1, registers.esr_el1)?;
        self.set_sys_reg(SysRegister::FarEl1, registers.far_el1)?;
        self.set_sys_reg(SysRegister::ParEl1, registers.par_el1)?;
        self.set_sys_reg(SysRegister::MairEl1, registers.mair_el1)?;
        self.set_sys_reg(SysRegister::AmairEl1, registers.amair_el1)?;
        self.set_sys_reg(SysRegister::VbarEl1, registers.vbar_el1)?;
        self.set_sys_reg(SysRegister::ContextidrEl1, registers.contextidr_el1)?;
        self.set_sys_reg(SysRegister::TpidrEl1, registers.tpidr_el1)?;
        self.set_sys_reg(SysRegister::ScxtnumEl1, registers.scxtnum_el1)?;
        self.set_sys_reg(SysRegister::CntkctlEl1, registers.cntkctl_el1)?;
        self.set_sys_reg(SysRegister::CsselrEl1, registers.csselr_el1)?;
        self.set_sys_reg(SysRegister::TpidrEl0, registers.tpidr_el0)?;
        self.set_sys_reg(SysRegister::TpidrroEl0, registers.tpidrro_el0)?;
        self.set_sys_reg(SysRegister::Tpidr2El0, registers.tpidr2_el0)?;
        self.set_sys_reg(SysRegister::ScxtnumEl0, registers.scxtnum_el0)?;
        self.set_sys_reg(SysRegister::CntvCtlEl0, registers.cntv_ctl_el0)?;
        self.set_sys_reg(SysRegister::CntvCvalEl0, registers.cntv_cval_el0)?;
        self.set_sys_reg(SysRegister::SpEl1, registers.sp_el1)?;
        self.set_sys_reg(SysRegister::CntpCtlEl0, registers.cntp_ctl_el0)?;
        self.set_sys_reg(SysRegister::CntpCvalEl0, registers.cntp_cval_el0)?;
        self.set_sys_reg(SysRegister::CntpTvalEl0, registers.cntp_tval_el0)?;
        self.set_sys_reg(SysRegister::CnthctlEl2, registers.cnthctl_el2)?;
        self.set_sys_reg(SysRegister::CnthpCtlEl2, registers.cnthp_ctl_el2)?;
        self.set_sys_reg(SysRegister::CnthpCvalEl2, registers.cnthp_cval_el2)?;
        self.set_sys_reg(SysRegister::CnthpTvalEl2, registers.cnthp_tval_el2)?;
        self.set_sys_reg(SysRegister::CntvoffEl2, registers.cntvoff_el2)?;
        self.set_sys_reg(SysRegister::CptrEl2, registers.cptr_el2)?;
        self.set_sys_reg(SysRegister::ElrEl2, registers.elr_el2)?;
        self.set_sys_reg(SysRegister::EsrEl2, registers.esr_el2)?;
        self.set_sys_reg(SysRegister::FarEl2, registers.far_el2)?;

        // self.set_sys_reg(SysRegister::HcrEl2, registers.hcr_el2)?; // TODO: stuck while booting, why?

        self.set_sys_reg(SysRegister::HpfarEl2, registers.hpfar_el2)?;
        self.set_sys_reg(SysRegister::MairEl2, registers.mair_el2)?;
        // self.set_sys_reg(SysRegister::MdcrEl2, registers.mdcr_el2)?; // mask
        self.set_sys_reg(SysRegister::SctlrEl2, registers.sctlr_el2)?;
        self.set_sys_reg(SysRegister::SpsrEl2, registers.spsr_el2)?;
        self.set_sys_reg(SysRegister::SpEl2, registers.sp_el2)?;
        self.set_sys_reg(SysRegister::TcrEl2, registers.tcr_el2)?;
        self.set_sys_reg(SysRegister::TpidrEl2, registers.tpidr_el2)?;
        self.set_sys_reg(SysRegister::Ttbr0El2, registers.ttbr0_el2)?;
        self.set_sys_reg(SysRegister::Ttbr1El2, registers.ttbr1_el2)?;
        self.set_sys_reg(SysRegister::VbarEl2, registers.vbar_el2)?;
        self.set_sys_reg(SysRegister::VmpidrEl2, registers.vmpidr_el2)?;
        self.set_sys_reg(SysRegister::VpidrEl2, registers.vpidr_el2)?;
        self.set_sys_reg(SysRegister::VtcrEl2, registers.vtcr_el2)?;
        self.set_sys_reg(SysRegister::VttbrEl2, registers.vttbr_el2)?;

        Ok(())
    }

    fn get_core_reg(&mut self, reg: CoreRegister) -> Result<u64, VcpuError> {
        let mut value = 0;

        match reg.into() {
            HvpReg::CoreReg(reg) => hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, reg, &mut value))?,
            HvpReg::SysReg(reg) => {
                hv_unsafe_call!(hv_vcpu_get_sys_reg(self.vcpu, reg, &mut value))?
            }
        }

        Ok(value)
    }

    fn set_core_reg(&mut self, reg: CoreRegister, value: u64) -> Result<(), VcpuError> {
        match reg.into() {
            HvpReg::CoreReg(reg) => {
                hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, reg, value)).map_err(Into::into)
            }
            HvpReg::SysReg(reg) => {
                hv_unsafe_call!(hv_vcpu_set_sys_reg(self.vcpu, reg, value)).map_err(Into::into)
            }
        }
    }

    fn get_fp_reg(&mut self, reg: FpRegister) -> Result<u128, VcpuError> {
        let mut value = 0;

        hv_unsafe_call!(hv_vcpu_get_simd_fp_reg(self.vcpu, reg.into(), &mut value))?;

        Ok(value)
    }

    fn set_fp_reg(&mut self, reg: FpRegister, value: u128) -> Result<(), VcpuError> {
        hv_unsafe_call!(hv_vcpu_set_simd_fp_reg(self.vcpu, reg.into(), value))?;

        Ok(())
    }

    fn get_sys_reg(&self, reg: SysRegister) -> Result<u64, VcpuError> {
        let mut value = 0;

        hv_unsafe_call!(hv_vcpu_get_sys_reg(self.vcpu, reg.into(), &mut value))?;

        Ok(value)
    }

    fn set_sys_reg(&mut self, reg: SysRegister, value: u64) -> Result<(), VcpuError> {
        hv_unsafe_call!(hv_vcpu_set_sys_reg(self.vcpu, reg.into(), value)).map_err(Into::into)
    }

    fn translate_gva_to_gpa(&self, gva: u64) -> Result<Option<u64>, VcpuError> {
        let tcr_el1 = || Ok(TcrEl1::from(self.get_sys_reg(SysRegister::TcrEl1)?));
        let ttbr1_el1 = || Ok(Ttbr1El1::from(self.get_sys_reg(SysRegister::Ttbr1El1)?));
        let id_aa64mmfr0_el1 = || {
            Ok(IdAa64mmfr0El1::from(
                self.get_sys_reg(SysRegister::IdAa64mmfr0El1)?,
            ))
        };
        translate_gva_to_gpa(&self.mm, tcr_el1, ttbr1_el1, id_aa64mmfr0_el1, gva)
    }
}

fn handle_command(
    running: &AtomicBool,
    hvp_vcpu_handler: Arc<Mutex<HvpVcpuInternal>>,
    cmd: VcpuCommand,
) -> Result<VcpuCommandResponse, VcpuError> {
    match cmd {
        VcpuCommand::ReadRegisters => {
            let mut handler = hvp_vcpu_handler.lock().unwrap();

            let registers = handler.read_registers()?;

            Ok(VcpuCommandResponse::Registers(Box::new(registers)))
        }
        VcpuCommand::WriteRegisters(registers) => {
            let mut handler: std::sync::MutexGuard<'_, HvpVcpuInternal> =
                hvp_vcpu_handler.lock().unwrap();

            handler.write_registers(registers)?;

            Ok(VcpuCommandResponse::Empty)
        }
        VcpuCommand::ReadCoreRegisters => {
            let mut handler = hvp_vcpu_handler.lock().unwrap();

            let registers = handler.read_core_registers()?;

            Ok(VcpuCommandResponse::CoreRegisters(Box::new(registers)))
        }
        VcpuCommand::WriteCoreRegisters(registers) => {
            let mut handler = hvp_vcpu_handler.lock().unwrap();

            handler.write_core_registers(registers)?;

            Ok(VcpuCommandResponse::Empty)
        }
        VcpuCommand::TranslateGvaToGpa(gva) => {
            let handler = hvp_vcpu_handler.lock().unwrap();

            let gpa = handler.translate_gva_to_gpa(gva)?;

            Ok(VcpuCommandResponse::TranslateGvaToGpa(gpa))
        }
        VcpuCommand::Resume => {
            running.store(true, Ordering::Release);

            Ok(VcpuCommandResponse::Empty)
        }
    }
}

fn handle_command_and_send_response(
    running: &AtomicBool,
    hvp_vcpu_handler: Arc<Mutex<HvpVcpuInternal>>,
    cmd: VcpuCommandRequest,
) {
    if let Err(_err) = match handle_command(running, hvp_vcpu_handler, cmd.cmd) {
        Ok(resp) => cmd.response.send(resp),
        Err(err) => cmd.response.send(VcpuCommandResponse::Err(err)),
    } {
        error!("Failed to send response of vcpu command");
    }
}

pub struct HvpVcpu {
    vcpu_id: usize,
    handler: Arc<Mutex<HvpVcpuInternal>>,
    command_tx: Sender<VcpuCommandRequest>,
    is_running: Arc<AtomicBool>,
    // TODO: handle gracefully shutdown
    #[allow(dead_code)]
    join_handler: JoinHandle<Result<(), VcpuError>>,
}

impl HvpVcpu {
    pub fn new(
        vcpu_id: usize,
        mm: Arc<MemoryAddressSpace>,
        vm_exit_handler: Arc<dyn VmExit>,
    ) -> Result<Self, VcpuError> {
        let (handler_tx, handler_rx) = std::sync::mpsc::channel();
        let (command_tx, mut command_rx) = tokio::sync::mpsc::channel(8);
        let is_running = Arc::new(AtomicBool::new(false));

        let is_running_cloned = is_running.clone();
        let join_handler = std::thread::spawn(move || -> Result<(), VcpuError> {
            let is_running = is_running_cloned;

            let mut vcpu = 0;
            let mut exit = null_mut() as *const hv_vcpu_exit_t;
            hv_unsafe_call!(hv_vcpu_create(&mut vcpu, &mut exit, null_mut()))?;

            let hvp_vcpu_handler = Arc::new(Mutex::new(HvpVcpuInternal { vcpu, mm }));

            handler_tx.send(hvp_vcpu_handler.clone()).unwrap();

            loop {
                {
                    // If there is pending command, handle it first before running the vCPU.
                    match command_rx.try_recv() {
                        Ok(cmd) => {
                            handle_command_and_send_response(
                                &is_running,
                                hvp_vcpu_handler.clone(),
                                cmd,
                            );
                        }
                        Err(TryRecvError::Empty) => (),
                        Err(TryRecvError::Disconnected) => {
                            return Err(VcpuError::VcpuCommandDisconnected);
                        }
                    }
                }

                {
                    // If vcpu is running, run it and handle vm exit.
                    if is_running.load(Ordering::Acquire) {
                        hv_unsafe_call!(hv_vcpu_run(vcpu))?;

                        let exit_reason = to_vm_exit(vcpu, unsafe { *exit })?;

                        let mut hvp_vcpu_handler = hvp_vcpu_handler.lock().unwrap();

                        match handle_vm_exit(
                            &mut *hvp_vcpu_handler,
                            exit_reason,
                            vm_exit_handler.as_ref(),
                        )? {
                            HandleVmExitResult::Canceled => {
                                is_running.store(false, Ordering::Release)
                            }
                            HandleVmExitResult::Continue => (),
                            HandleVmExitResult::NextInstruction => {
                                let pc = hvp_vcpu_handler.get_core_reg(CoreRegister::PC)?;
                                hvp_vcpu_handler.set_core_reg(CoreRegister::PC, pc + 4)?;
                            }
                        }

                        continue;
                    }
                }

                {
                    // Otherwise, just wait for command.
                    command_rx
                        .blocking_recv()
                        .ok_or(VcpuError::VcpuCommandDisconnected)
                        .map(|cmd| {
                            handle_command_and_send_response(
                                &is_running,
                                hvp_vcpu_handler.clone(),
                                cmd,
                            )
                        })?;
                }
            }
        });

        let handler = handler_rx.recv().unwrap();

        Ok(HvpVcpu {
            vcpu_id,
            handler,
            command_tx,
            is_running,
            join_handler,
        })
    }
}

impl HypervisorVcpu for HvpVcpu {
    fn vcpu_id(&self) -> usize {
        self.vcpu_id
    }

    fn command_tx(&self) -> WeakSender<VcpuCommandRequest> {
        self.command_tx.downgrade()
    }

    fn tick(&self) -> Result<(), VcpuError> {
        if !self.is_running.load(Ordering::Acquire) {
            return Ok(());
        }

        let handlers = [self.handler.lock().unwrap().vcpu];

        hv_unsafe_call!(hv_vcpus_exit(
            handlers.as_ptr(),
            handlers.len().try_into().unwrap()
        ))?;

        Ok(())
    }
}
