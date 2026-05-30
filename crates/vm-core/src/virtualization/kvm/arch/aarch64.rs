use std::hint::black_box;
use std::sync::atomic::{AtomicBool, Ordering};

use vm_mm::manager::MemoryAddressSpace;

use crate::arch::aarch64::vcpu::AArch64Vcpu;
use crate::arch::aarch64::vcpu::reg::{CoreRegister, FpRegister, SysRegister};
use crate::virtualization::kvm::vcpu::KvmVcpuInternal;
use crate::virtualization::vcpu::command::{VcpuCommand, VcpuCommandResponse};
use crate::virtualization::vcpu::error::VcpuError;

impl<'a> AArch64Vcpu for KvmVcpuInternal<'a> {
    fn get_core_reg(&self, _reg: CoreRegister) -> Result<u64, VcpuError> {
        black_box(self.vcpu_fd);

        todo!()
    }

    fn set_core_reg(&mut self, _reg: CoreRegister, _value: u64) -> Result<(), VcpuError> {
        todo!()
    }

    fn get_fp_reg(&self, _reg: FpRegister) -> Result<u128, VcpuError> {
        todo!()
    }

    fn set_fp_reg(&mut self, _reg: FpRegister, _value: u128) -> Result<(), VcpuError> {
        todo!()
    }

    fn get_sys_reg(&self, _reg: SysRegister) -> Result<u64, VcpuError> {
        todo!()
    }

    fn set_sys_reg(&mut self, _reg: SysRegister, _value: u64) -> Result<(), VcpuError> {
        todo!()
    }

    fn mm(&self) -> &MemoryAddressSpace {
        todo!()
    }
}

impl<'a> KvmVcpuInternal<'a> {
    pub fn handle_command(
        &mut self,
        is_running: &AtomicBool,
        cmd: VcpuCommand,
    ) -> Result<VcpuCommandResponse, VcpuError> {
        match cmd {
            VcpuCommand::ReadRegisters => {
                let registers = self.read_registers()?;

                Ok(VcpuCommandResponse::Registers(Box::new(registers)))
            }
            VcpuCommand::WriteRegisters(_) => todo!(),
            VcpuCommand::ReadCoreRegisters => todo!(),
            VcpuCommand::WriteCoreRegisters(_) => todo!(),
            VcpuCommand::Save => todo!(),
            VcpuCommand::Load(_) => todo!(),
            VcpuCommand::TranslateGvaToGpa(_) => todo!(),
            VcpuCommand::Resume => {
                is_running.store(true, Ordering::Release);

                Ok(VcpuCommandResponse::Empty)
            }
        }
    }
}
