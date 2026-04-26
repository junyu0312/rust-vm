use std::ptr::null_mut;
use std::sync::Arc;
use std::sync::Mutex;

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
    running: &mut bool,
    hvp_vcpu_handler: Arc<Mutex<HvpVcpuInternal>>,
    cmd: VcpuCommandRequest,
) -> Result<(), VcpuError> {
    match cmd.cmd {
        VcpuCommand::ReadRegisters => {
            let mut handler = hvp_vcpu_handler.lock().unwrap();

            let registers = handler.read_registers()?;

            cmd.response
                .send(VcpuCommandResponse::Registers(Box::new(registers)))
                .map_err(|_| VcpuError::VcpuCommandDisconnected)?;
        }
        VcpuCommand::WriteRegisters(registers) => {
            let mut handler = hvp_vcpu_handler.lock().unwrap();

            handler.write_registers(registers)?;

            cmd.response
                .send(VcpuCommandResponse::Empty)
                .map_err(|_| VcpuError::VcpuCommandDisconnected)?;
        }
        VcpuCommand::ReadCoreRegisters => {
            let mut handler = hvp_vcpu_handler.lock().unwrap();

            let registers = handler.read_core_registers()?;

            cmd.response
                .send(VcpuCommandResponse::CoreRegisters(Box::new(registers)))
                .map_err(|_| VcpuError::VcpuCommandDisconnected)?;
        }
        VcpuCommand::WriteCoreRegisters(registers) => {
            let mut handler = hvp_vcpu_handler.lock().unwrap();

            handler.write_core_registers(registers)?;

            cmd.response
                .send(VcpuCommandResponse::Empty)
                .map_err(|_| VcpuError::VcpuCommandDisconnected)?;
        }
        VcpuCommand::TranslateGvaToGpa(gva) => {
            let handler = hvp_vcpu_handler.lock().unwrap();

            let gpa = handler.translate_gva_to_gpa(gva)?;

            cmd.response
                .send(VcpuCommandResponse::TranslateGvaToGpa(gpa))
                .map_err(|_| VcpuError::VcpuCommandDisconnected)?;
        }
        VcpuCommand::Resume => {
            *running = true;
            cmd.response
                .send(VcpuCommandResponse::Empty)
                .map_err(|_| VcpuError::VcpuCommandDisconnected)?;
        }
        VcpuCommand::Pause => {
            *running = false;
            cmd.response
                .send(VcpuCommandResponse::Empty)
                .map_err(|_| VcpuError::VcpuCommandDisconnected)?;
        }
    }

    Ok(())
}

pub struct HvpVcpu {
    vcpu_id: usize,
    handler: Arc<Mutex<HvpVcpuInternal>>,
    command_tx: Sender<VcpuCommandRequest>,
}

impl HvpVcpu {
    pub fn new(
        vcpu_id: usize,
        mm: Arc<MemoryAddressSpace>,
        vm_exit_handler: Arc<dyn VmExit>,
    ) -> Result<Self, VcpuError> {
        let (handler_tx, handler_rx) = std::sync::mpsc::channel();
        let (command_tx, mut command_rx) = tokio::sync::mpsc::channel(8);

        let _join_handler = std::thread::spawn(move || -> Result<(), VcpuError> {
            let mut vcpu = 0;
            let mut exit = null_mut() as *const hv_vcpu_exit_t;
            hv_unsafe_call!(hv_vcpu_create(&mut vcpu, &mut exit, null_mut()))?;

            let hvp_vcpu_handler = Arc::new(Mutex::new(HvpVcpuInternal { vcpu, mm }));

            handler_tx.send(hvp_vcpu_handler.clone()).unwrap();

            let mut running = false;

            loop {
                if running {
                    hv_unsafe_call!(hv_vcpu_run(vcpu))?;

                    let exit_reason = to_vm_exit(vcpu, unsafe { *exit })?;

                    let mut hvp_vcpu_handler = hvp_vcpu_handler.lock().unwrap();

                    match handle_vm_exit(
                        &mut *hvp_vcpu_handler,
                        exit_reason,
                        vm_exit_handler.as_ref(),
                    )? {
                        HandleVmExitResult::Continue => (),
                        HandleVmExitResult::NextInstruction => {
                            let pc = hvp_vcpu_handler.get_core_reg(CoreRegister::PC)?;
                            hvp_vcpu_handler.set_core_reg(CoreRegister::PC, pc + 4)?;
                        }
                    }
                }

                let cmd: VcpuCommandRequest;
                if running {
                    match command_rx.try_recv() {
                        Ok(command) => cmd = command,
                        Err(TryRecvError::Empty) => continue,
                        Err(TryRecvError::Disconnected) => {
                            return Err(VcpuError::VcpuCommandDisconnected);
                        }
                    }
                } else {
                    cmd = command_rx
                        .blocking_recv()
                        .ok_or(VcpuError::VcpuCommandDisconnected)?;
                }

                if let Err(err) = handle_command(&mut running, hvp_vcpu_handler.clone(), cmd) {
                    error!(?err, "Failed to handle cmd")
                }
            }
        });

        let handler = handler_rx.recv().unwrap();

        Ok(HvpVcpu {
            vcpu_id,
            handler,
            command_tx,
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

    fn tick(&mut self) -> Result<(), VcpuError> {
        let handlers = [self.handler.lock().unwrap().vcpu];

        hv_unsafe_call!(hv_vcpus_exit(
            handlers.as_ptr(),
            handlers.len().try_into().unwrap()
        ))?;

        Ok(())
    }
}
