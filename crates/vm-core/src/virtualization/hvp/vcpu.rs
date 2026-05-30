use std::collections::BTreeMap;
use std::ptr::null_mut;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;

use applevisor_sys::hv_error_t;
use applevisor_sys::hv_gic_get_icc_reg;
use applevisor_sys::hv_gic_get_ich_reg;
use applevisor_sys::hv_gic_get_icv_reg;
use applevisor_sys::hv_gic_get_redistributor_reg;
use applevisor_sys::hv_gic_set_icc_reg;
use applevisor_sys::hv_gic_set_ich_reg;
use applevisor_sys::hv_gic_set_icv_reg;
use applevisor_sys::hv_gic_set_redistributor_reg;
use applevisor_sys::hv_reg_t;
use applevisor_sys::hv_simd_fp_reg_t;
use applevisor_sys::hv_sys_reg_t;
use applevisor_sys::hv_vcpu_create;
use applevisor_sys::hv_vcpu_exit_t;
use applevisor_sys::hv_vcpu_get_reg;
use applevisor_sys::hv_vcpu_get_simd_fp_reg;
use applevisor_sys::hv_vcpu_get_sys_reg;
use applevisor_sys::hv_vcpu_get_vtimer_mask;
use applevisor_sys::hv_vcpu_get_vtimer_offset;
use applevisor_sys::hv_vcpu_run;
use applevisor_sys::hv_vcpu_set_reg;
use applevisor_sys::hv_vcpu_set_simd_fp_reg;
use applevisor_sys::hv_vcpu_set_sys_reg;
use applevisor_sys::hv_vcpu_set_vtimer_mask;
use applevisor_sys::hv_vcpu_set_vtimer_offset;
use applevisor_sys::hv_vcpu_t;
use applevisor_sys::hv_vcpus_exit;
use strum::IntoEnumIterator;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::WeakSender;
use tokio::sync::mpsc::error::TryRecvError;
use tracing::error;
use vm_mm::manager::MemoryAddressSpace;

use crate::arch::aarch64::vcpu::AArch64Vcpu;
use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::FpRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::arch::aarch64::vm_exit::HandleVmExitResult;
use crate::arch::aarch64::vm_exit::handle_vm_exit;
use crate::cpu::vm_exit::VmExit;
use crate::virtualization::hvp::hv_unsafe_call;
use crate::virtualization::hvp::vcpu::register::AppleHypervisorCoreRegister;
use crate::virtualization::hvp::vcpu::register::AppleHypervisorFpRegister;
use crate::virtualization::hvp::vcpu::register::AppleHypervisorGicIccReg;
use crate::virtualization::hvp::vcpu::register::AppleHypervisorGicIchReg;
use crate::virtualization::hvp::vcpu::register::AppleHypervisorGicIcvReg;
use crate::virtualization::hvp::vcpu::register::AppleHypervisorGicRedistributorReg;
use crate::virtualization::hvp::vcpu::register::AppleHypervisorSysRegister;
use crate::virtualization::hvp::vcpu::register::AppleHypervisorVcpuRegisters;
use crate::virtualization::hvp::vcpu::vm_exit::to_vm_exit;
use crate::virtualization::vcpu::HypervisorVcpu;
use crate::virtualization::vcpu::command::VcpuCommand;
use crate::virtualization::vcpu::command::VcpuCommandRequest;
use crate::virtualization::vcpu::command::VcpuCommandResponse;
use crate::virtualization::vcpu::error::VcpuError;

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
mod register;
mod vm_exit;

struct HvpVcpuInternal {
    // handler for apple hypervisor vcpu
    vcpu: hv_vcpu_t,
    mm: Arc<MemoryAddressSpace>,
}

impl HvpVcpuInternal {
    fn save(&self) -> Result<Vec<u8>, VcpuError> {
        let gic_redistributor = {
            let mut map = BTreeMap::new();
            for reg in AppleHypervisorGicRedistributorReg::iter() {
                let mut value = 0;
                if hv_unsafe_call!(hv_gic_get_redistributor_reg(
                    self.vcpu,
                    reg.into(),
                    &mut value
                ))
                .inspect_err(|err| error!(?reg, ?err))
                .is_ok()
                {
                    map.insert(reg, value);
                }
            }

            map
        };
        let gic_icc = {
            let mut map = BTreeMap::new();
            for reg in AppleHypervisorGicIccReg::iter() {
                let mut value = 0;
                if hv_unsafe_call!(hv_gic_get_icc_reg(self.vcpu, reg.into(), &mut value))
                    .inspect_err(|err| error!(?reg, ?err))
                    .is_ok()
                {
                    map.insert(reg, value);
                }
            }

            map
        };
        let gic_ich = {
            let mut map = BTreeMap::new();
            for reg in AppleHypervisorGicIchReg::iter() {
                let mut value = 0;
                if hv_unsafe_call!(hv_gic_get_ich_reg(self.vcpu, reg.into(), &mut value))
                    .inspect_err(|err| error!(?reg, ?err))
                    .is_ok()
                {
                    map.insert(reg, value);
                }
            }

            map
        };
        let gic_icv = {
            let mut map = BTreeMap::new();
            for reg in AppleHypervisorGicIcvReg::iter() {
                let mut value = 0;
                if hv_unsafe_call!(hv_gic_get_icv_reg(self.vcpu, reg.into(), &mut value))
                    .inspect_err(|err| error!(?reg, ?err))
                    .is_ok()
                {
                    map.insert(reg, value);
                }
            }

            map
        };
        let core = {
            let mut map = BTreeMap::new();
            for reg in AppleHypervisorCoreRegister::iter() {
                let mut value = 0;
                if hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, reg.into(), &mut value))
                    .inspect_err(|err| error!(?reg, ?err))
                    .is_ok()
                {
                    map.insert(reg, value);
                }
            }

            map
        };
        let sys = {
            let mut map = BTreeMap::new();
            for reg in AppleHypervisorSysRegister::iter() {
                let mut value = 0;
                if hv_unsafe_call!(hv_vcpu_get_sys_reg(self.vcpu, reg.into(), &mut value))
                    .inspect_err(|err| error!(?reg, ?err))
                    .is_ok()
                {
                    map.insert(reg, value);
                }
            }

            map
        };
        let fp = {
            let mut map = BTreeMap::new();
            for reg in AppleHypervisorFpRegister::iter() {
                let mut value = 0;
                if hv_unsafe_call!(hv_vcpu_get_simd_fp_reg(self.vcpu, reg.into(), &mut value))
                    .inspect_err(|err| error!(?reg, ?err))
                    .is_ok()
                {
                    map.insert(reg, value);
                }
            }

            map
        };

        let vtimer_is_masked = {
            let mut vtimer_is_masked = false;
            hv_unsafe_call!(hv_vcpu_get_vtimer_mask(self.vcpu, &mut vtimer_is_masked))?;
            vtimer_is_masked
        };

        let vtimer_offset = {
            let mut vtimer_offset = 0;
            hv_unsafe_call!(hv_vcpu_get_vtimer_offset(self.vcpu, &mut vtimer_offset))?;
            vtimer_offset
        };

        let snap = AppleHypervisorVcpuRegisters {
            gic_redistributor,
            gic_icc,
            gic_ich,
            gic_icv,
            core,
            sys,
            fp,
            vtimer_is_masked,
            vtimer_offset,
        };

        let buf = serde_json::to_vec(&snap).map_err(|err| VcpuError::Save(err.into()))?;

        Ok(buf)
    }

    fn load(&mut self, buf: Vec<u8>) -> Result<(), VcpuError> {
        let snap = serde_json::from_slice::<AppleHypervisorVcpuRegisters>(&buf)
            .map_err(|err| VcpuError::Save(err.into()))?;

        for (reg, value) in snap.gic_redistributor {
            let _ = hv_unsafe_call!(hv_gic_set_redistributor_reg(self.vcpu, reg.into(), value))
                .inspect_err(|err| error!(?reg, ?err));
        }

        for (reg, value) in snap.gic_icc {
            let _ = hv_unsafe_call!(hv_gic_set_icc_reg(self.vcpu, reg.into(), value))
                .inspect_err(|err| error!(?reg, ?err));
        }

        for (reg, value) in snap.gic_ich {
            let _ = hv_unsafe_call!(hv_gic_set_ich_reg(self.vcpu, reg.into(), value))
                .inspect_err(|err| error!(?reg, ?err));
        }

        for (reg, value) in snap.gic_icv {
            let _ = hv_unsafe_call!(hv_gic_set_icv_reg(self.vcpu, reg.into(), value))
                .inspect_err(|err| error!(?reg, ?err));
        }

        for (reg, value) in snap.core {
            let _ = hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, reg.into(), value))
                .inspect_err(|err| error!(?reg, ?err));
        }

        for (reg, value) in snap.sys {
            let _ = hv_unsafe_call!(hv_vcpu_set_sys_reg(self.vcpu, reg.into(), value))
                .inspect_err(|err| error!(?reg, ?err));
        }

        for (reg, value) in snap.fp {
            let _ = hv_unsafe_call!(hv_vcpu_set_simd_fp_reg(self.vcpu, reg.into(), value))
                .inspect_err(|err| error!(?reg, ?err));
        }

        let _ = hv_unsafe_call!(hv_vcpu_set_vtimer_mask(self.vcpu, snap.vtimer_is_masked))
            .inspect_err(|err| error!(?err, "failed to set vtimer_mask"));

        let _ = hv_unsafe_call!(hv_vcpu_set_vtimer_offset(self.vcpu, snap.vtimer_offset))
            .inspect_err(|err| error!(?err, "failed to set vtimer_offset"));

        Ok(())
    }
}

impl AArch64Vcpu for HvpVcpuInternal {
    fn get_core_reg(&self, reg: CoreRegister) -> Result<u64, VcpuError> {
        let mut value = 0;

        match reg {
            CoreRegister::X0 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X0, &mut value))?
            }
            CoreRegister::X1 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X1, &mut value))?
            }
            CoreRegister::X2 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X2, &mut value))?
            }
            CoreRegister::X3 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X3, &mut value))?
            }
            CoreRegister::X4 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X4, &mut value))?
            }
            CoreRegister::X5 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X5, &mut value))?
            }
            CoreRegister::X6 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X6, &mut value))?
            }
            CoreRegister::X7 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X7, &mut value))?
            }
            CoreRegister::X8 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X8, &mut value))?
            }
            CoreRegister::X9 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X9, &mut value))?
            }
            CoreRegister::X10 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X10, &mut value))?
            }
            CoreRegister::X11 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X11, &mut value))?
            }
            CoreRegister::X12 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X12, &mut value))?
            }
            CoreRegister::X13 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X13, &mut value))?
            }
            CoreRegister::X14 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X14, &mut value))?
            }
            CoreRegister::X15 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X15, &mut value))?
            }
            CoreRegister::X16 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X16, &mut value))?
            }
            CoreRegister::X17 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X17, &mut value))?
            }
            CoreRegister::X18 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X18, &mut value))?
            }
            CoreRegister::X19 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X19, &mut value))?
            }
            CoreRegister::X20 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X20, &mut value))?
            }
            CoreRegister::X21 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X21, &mut value))?
            }
            CoreRegister::X22 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X22, &mut value))?
            }
            CoreRegister::X23 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X23, &mut value))?
            }
            CoreRegister::X24 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X24, &mut value))?
            }
            CoreRegister::X25 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X25, &mut value))?
            }
            CoreRegister::X26 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X26, &mut value))?
            }
            CoreRegister::X27 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X27, &mut value))?
            }
            CoreRegister::X28 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X28, &mut value))?
            }
            CoreRegister::X29 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X29, &mut value))?
            }
            CoreRegister::X30 => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::X30, &mut value))?
            }
            CoreRegister::SP => {
                hv_unsafe_call!(hv_vcpu_get_sys_reg(
                    self.vcpu,
                    hv_sys_reg_t::SP_EL0,
                    &mut value
                ))? // TODO: SP_EL?
            }
            CoreRegister::PC => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::PC, &mut value))?
            }
            CoreRegister::PState => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::CPSR, &mut value))?
            }
            CoreRegister::Fpcr => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::FPCR, &mut value))?
            }
            CoreRegister::Fpsr => {
                hv_unsafe_call!(hv_vcpu_get_reg(self.vcpu, hv_reg_t::FPSR, &mut value))?
            }
        }

        Ok(value)
    }

    fn set_core_reg(&mut self, reg: CoreRegister, value: u64) -> Result<(), VcpuError> {
        match reg {
            CoreRegister::X0 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X0, value))?,
            CoreRegister::X1 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X1, value))?,
            CoreRegister::X2 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X2, value))?,
            CoreRegister::X3 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X3, value))?,
            CoreRegister::X4 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X4, value))?,
            CoreRegister::X5 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X5, value))?,
            CoreRegister::X6 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X6, value))?,
            CoreRegister::X7 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X7, value))?,
            CoreRegister::X8 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X8, value))?,
            CoreRegister::X9 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X9, value))?,
            CoreRegister::X10 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X10, value))?,
            CoreRegister::X11 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X11, value))?,
            CoreRegister::X12 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X12, value))?,
            CoreRegister::X13 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X13, value))?,
            CoreRegister::X14 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X14, value))?,
            CoreRegister::X15 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X15, value))?,
            CoreRegister::X16 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X16, value))?,
            CoreRegister::X17 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X17, value))?,
            CoreRegister::X18 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X18, value))?,
            CoreRegister::X19 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X19, value))?,
            CoreRegister::X20 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X20, value))?,
            CoreRegister::X21 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X21, value))?,
            CoreRegister::X22 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X22, value))?,
            CoreRegister::X23 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X23, value))?,
            CoreRegister::X24 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X24, value))?,
            CoreRegister::X25 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X25, value))?,
            CoreRegister::X26 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X26, value))?,
            CoreRegister::X27 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X27, value))?,
            CoreRegister::X28 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X28, value))?,
            CoreRegister::X29 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X29, value))?,
            CoreRegister::X30 => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::X30, value))?,
            CoreRegister::SP => {
                hv_unsafe_call!(hv_vcpu_set_sys_reg(self.vcpu, hv_sys_reg_t::SP_EL0, value))?
            } // TODO: SP_EL?
            CoreRegister::PC => hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::PC, value))?,
            CoreRegister::PState => {
                hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::CPSR, value))?
            }
            CoreRegister::Fpcr => {
                hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::FPCR, value))?
            }
            CoreRegister::Fpsr => {
                hv_unsafe_call!(hv_vcpu_set_reg(self.vcpu, hv_reg_t::FPSR, value))?
            }
        }

        Ok(())
    }

    fn get_fp_reg(&self, reg: FpRegister) -> Result<u128, VcpuError> {
        let mut value = 0;

        hv_unsafe_call!(hv_vcpu_get_simd_fp_reg(
            self.vcpu,
            hv_simd_fp_reg_t::from(AppleHypervisorFpRegister::from(reg)),
            &mut value
        ))?;

        Ok(value)
    }

    fn set_fp_reg(&mut self, reg: FpRegister, value: u128) -> Result<(), VcpuError> {
        hv_unsafe_call!(hv_vcpu_set_simd_fp_reg(
            self.vcpu,
            hv_simd_fp_reg_t::from(AppleHypervisorFpRegister::from(reg)),
            value
        ))?;

        Ok(())
    }

    fn get_sys_reg(&self, reg: SysRegister) -> Result<u64, VcpuError> {
        let mut value = 0;

        hv_unsafe_call!(hv_vcpu_get_sys_reg(
            self.vcpu,
            hv_sys_reg_t::from(AppleHypervisorSysRegister::try_from(reg)?),
            &mut value
        ))?;

        Ok(value)
    }

    fn set_sys_reg(&mut self, reg: SysRegister, value: u64) -> Result<(), VcpuError> {
        hv_unsafe_call!(hv_vcpu_set_sys_reg(
            self.vcpu,
            hv_sys_reg_t::from(AppleHypervisorSysRegister::try_from(reg)?),
            value
        ))?;

        Ok(())
    }

    fn mm(&self) -> &MemoryAddressSpace {
        &self.mm
    }
}

fn handle_command(
    running: &AtomicBool,
    hvp_vcpu_handler: Arc<Mutex<HvpVcpuInternal>>,
    cmd: VcpuCommand,
) -> Result<VcpuCommandResponse, VcpuError> {
    match cmd {
        VcpuCommand::ReadRegisters => {
            let handler = hvp_vcpu_handler.lock().unwrap();

            let registers = handler.read_registers()?;

            Ok(VcpuCommandResponse::Registers(Box::new(registers)))
        }
        VcpuCommand::WriteRegisters(registers) => {
            let mut handler = hvp_vcpu_handler.lock().unwrap();

            handler.write_registers(*registers)?;

            Ok(VcpuCommandResponse::Empty)
        }
        VcpuCommand::ReadCoreRegisters => {
            let handler = hvp_vcpu_handler.lock().unwrap();

            let registers = handler.read_core_registers()?;

            Ok(VcpuCommandResponse::CoreRegisters(Box::new(registers)))
        }
        VcpuCommand::WriteCoreRegisters(registers) => {
            let mut handler = hvp_vcpu_handler.lock().unwrap();

            handler.write_core_registers(*registers)?;

            Ok(VcpuCommandResponse::Empty)
        }
        VcpuCommand::Save => {
            let handler = hvp_vcpu_handler.lock().unwrap();

            let buf = handler.save()?;

            Ok(VcpuCommandResponse::Save(buf))
        }
        VcpuCommand::Load(buf) => {
            let mut handler = hvp_vcpu_handler.lock().unwrap();

            handler.load(buf)?;

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
    vcpu_id: u64,
    handler: Arc<Mutex<HvpVcpuInternal>>,
    command_tx: Sender<VcpuCommandRequest>,
    is_running: Arc<AtomicBool>,
    // TODO: handle gracefully shutdown
    #[allow(dead_code)]
    join_handler: JoinHandle<Result<(), VcpuError>>,
}

impl HvpVcpu {
    pub fn new(
        vcpu_id: u64,
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

                            continue;
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

                        let mut hvp_vcpu_handler = hvp_vcpu_handler.lock().unwrap();

                        let exit_reason = to_vm_exit(&*hvp_vcpu_handler, unsafe { *exit })?;

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
    fn vcpu_id(&self) -> u64 {
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
