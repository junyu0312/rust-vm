use std::ptr::null_mut;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;

use applevisor_sys::hv_error_t;
use applevisor_sys::hv_vcpu_create;
use applevisor_sys::hv_vcpu_exit_t;
use applevisor_sys::hv_vcpu_get_reg;
use applevisor_sys::hv_vcpu_get_sys_reg;
use applevisor_sys::hv_vcpu_run;
use applevisor_sys::hv_vcpu_set_reg;
use applevisor_sys::hv_vcpu_set_sys_reg;
use applevisor_sys::hv_vcpu_t;
use applevisor_sys::hv_vcpus_exit;
use async_trait::async_trait;

use crate::arch::aarch64::register::AArch64Registers;
use crate::arch::aarch64::vcpu::AArch64Vcpu;
use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::arch::aarch64::vm_exit::HandleVmExitResult;
use crate::arch::aarch64::vm_exit::handle_vm_exit;
use crate::cpu::error::VcpuError;
use crate::cpu::vm_exit::VmExit;
use crate::virtualization::hvp::hv_unsafe_call;
use crate::virtualization::hvp::vcpu::register::HvpReg;
use crate::virtualization::hvp::vcpu::vm_exit::to_vm_exit;
use crate::virtualization::vcpu::HypervisorVcpu;
use crate::virtualization::vcpu::command::VcpuCommand;
use crate::virtualization::vcpu::command::VcpuCommandRequest;
use crate::virtualization::vcpu::command::VcpuCommandResponse;

mod register;
mod vm_exit;

struct HvpVcpuInternal {
    /// handler for apple hypervisor vcpu
    vcpu: hv_vcpu_t,
}

impl AArch64Vcpu for HvpVcpuInternal {
    fn read_registers(&mut self) -> Result<AArch64Registers, VcpuError> {
        Ok(AArch64Registers {
            x0: self.get_core_reg(CoreRegister::X0)?,
            x1: self.get_core_reg(CoreRegister::X1)?,
            x2: self.get_core_reg(CoreRegister::X2)?,
            x3: self.get_core_reg(CoreRegister::X3)?,
            pc: self.get_core_reg(CoreRegister::PC)?,
            pstate: self.get_core_reg(CoreRegister::PState)?,

            mpidr_el1: self.get_sys_reg(SysRegister::MpidrEl1)?,
            sctlr_el1: self.get_sys_reg(SysRegister::SctlrEl1)?,
            cnthctl_el2: self.get_sys_reg(SysRegister::CnthctlEl2)?,
        })
    }

    fn write_registers(&mut self, registers: AArch64Registers) -> Result<(), VcpuError> {
        self.set_core_reg(CoreRegister::X0, registers.x0)?;
        self.set_core_reg(CoreRegister::X1, registers.x1)?;
        self.set_core_reg(CoreRegister::X2, registers.x2)?;
        self.set_core_reg(CoreRegister::X3, registers.x3)?;
        self.set_core_reg(CoreRegister::PC, registers.pc)?;
        self.set_core_reg(CoreRegister::PState, registers.pstate)?;

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

    fn get_sys_reg(&mut self, reg: SysRegister) -> Result<u64, VcpuError> {
        let mut value = 0;

        hv_unsafe_call!(hv_vcpu_get_sys_reg(self.vcpu, reg.into(), &mut value))?;

        Ok(value)
    }

    fn set_sys_reg(&mut self, reg: SysRegister, value: u64) -> Result<(), VcpuError> {
        hv_unsafe_call!(hv_vcpu_set_sys_reg(self.vcpu, reg.into(), value)).map_err(Into::into)
    }
}

pub struct HvpVcpu {
    vcpu_id: usize,
    handler: Arc<Mutex<HvpVcpuInternal>>,
    command_tx: Sender<VcpuCommandRequest>,
}

impl HvpVcpu {
    pub fn new(vcpu_id: usize, vm_exit_handler: Arc<dyn VmExit>) -> Result<Self, VcpuError> {
        let (handler_tx, handler_rx) = mpsc::channel();
        let (command_tx, command_rx) = mpsc::channel();

        let _join_handler = std::thread::spawn(move || -> Result<(), VcpuError> {
            let mut vcpu = 0;
            let mut exit = null_mut() as *const hv_vcpu_exit_t;
            hv_unsafe_call!(hv_vcpu_create(&mut vcpu, &mut exit, null_mut()))?;
            println!("vcpu_id {} map to vcpu: {}", vcpu_id, vcpu);

            let hvp_vcpu_handler = Arc::new(Mutex::new(HvpVcpuInternal { vcpu }));

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
                        .recv()
                        .map_err(|_| VcpuError::VcpuCommandDisconnected)?;
                }

                match cmd.cmd {
                    VcpuCommand::ReadRegisters => {
                        let mut handler = hvp_vcpu_handler.lock().unwrap();

                        let registers = handler.read_registers()?;

                        cmd.response
                            .send(VcpuCommandResponse::Registers(registers))
                            .map_err(|_| VcpuError::VcpuCommandDisconnected)?;
                    }
                    VcpuCommand::WriteRegisters(registers) => {
                        let mut handler = hvp_vcpu_handler.lock().unwrap();

                        handler.write_registers(registers)?;

                        cmd.response
                            .send(VcpuCommandResponse::Empty)
                            .map_err(|_| VcpuError::VcpuCommandDisconnected)?;
                    }
                    VcpuCommand::Pause => {
                        running = false;
                        cmd.response
                            .send(VcpuCommandResponse::Empty)
                            .map_err(|_| VcpuError::VcpuCommandDisconnected)?;
                    }
                    VcpuCommand::Resume => {
                        running = true;
                        cmd.response
                            .send(VcpuCommandResponse::Empty)
                            .map_err(|_| VcpuError::VcpuCommandDisconnected)?;
                    }
                }
            }
        });

        let handler = handler_rx
            .recv()
            .map_err(|err| VcpuError::FailedToCreateVcpu(Box::new(err)))?;

        Ok(HvpVcpu {
            vcpu_id,
            handler,
            command_tx,
        })
    }
}

#[async_trait]
impl HypervisorVcpu for HvpVcpu {
    async fn read_reigsters(&mut self) -> Result<AArch64Registers, VcpuError> {
        let (cmd, rx) = VcpuCommandRequest::new(VcpuCommand::ReadRegisters);

        self.command_tx
            .send(cmd)
            .map_err(|_| VcpuError::VcpuCommandDisconnected)?;

        match rx.await.map_err(|_| VcpuError::VcpuCommandDisconnected)? {
            VcpuCommandResponse::Empty => unreachable!(),
            VcpuCommandResponse::Registers(registers) => return Ok(registers),
        }
    }

    async fn write_registers(&mut self, registers: AArch64Registers) -> Result<(), VcpuError> {
        let (cmd, rx) = VcpuCommandRequest::new(VcpuCommand::WriteRegisters(registers));

        self.command_tx
            .send(cmd)
            .map_err(|_| VcpuError::VcpuCommandDisconnected)?;

        rx.await.map_err(|_| VcpuError::VcpuCommandDisconnected)?;

        Ok(())
    }

    async fn resume(&mut self) -> Result<(), VcpuError> {
        let (cmd, rx) = VcpuCommandRequest::new(VcpuCommand::Resume);

        self.command_tx
            .send(cmd)
            .map_err(|_| VcpuError::VcpuCommandDisconnected)?;

        rx.await.map_err(|_| VcpuError::VcpuCommandDisconnected)?;

        Ok(())
    }

    async fn pause(&mut self) -> Result<(), VcpuError> {
        let (cmd, rx) = VcpuCommandRequest::new(VcpuCommand::Pause);

        self.command_tx
            .send(cmd)
            .map_err(|_| VcpuError::VcpuCommandDisconnected)?;

        let handlers = [self.handler.lock().unwrap().vcpu];

        hv_unsafe_call!(hv_vcpus_exit(
            handlers.as_ptr(),
            handlers.len().try_into().unwrap()
        ))?;

        rx.await.map_err(|_| VcpuError::VcpuCommandDisconnected)?;

        Ok(())
    }
}
