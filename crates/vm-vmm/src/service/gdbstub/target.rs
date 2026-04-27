use std::sync::Arc;

use gdbstub::arch::Arch;
use gdbstub::common::Signal;
use gdbstub::common::Tid;
use gdbstub::target::Target;
use gdbstub::target::TargetError;
use gdbstub::target::TargetResult;
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::base::multithread::MultiThreadBase;
use gdbstub::target::ext::base::multithread::MultiThreadResume;
use gdbstub::target::ext::base::multithread::MultiThreadResumeOps;
use tokio::sync::mpsc;
use tracing::error;

use crate::service::gdbstub::GdbStubArch;
use crate::service::gdbstub::command::GdbStubCommand;
use crate::service::gdbstub::command::GdbStubCommandResponse;
use crate::service::gdbstub::error::VmGdbStubError;
use crate::vmm::handler::VmmCommand;

fn vcpu_id_to_tid(vcpu_id: usize) -> Result<Tid, VmGdbStubError> {
    Tid::new(vcpu_id + 1).ok_or(VmGdbStubError::InvalidTid)
}

fn tid_to_vcpu_id(tid: Tid) -> usize {
    tid.get() - 1
}

pub struct VmGdbStubTarget {
    tx: Arc<mpsc::Sender<VmmCommand>>,
}

impl VmGdbStubTarget {
    pub fn new(tx: Arc<mpsc::Sender<VmmCommand>>) -> VmGdbStubTarget {
        VmGdbStubTarget { tx }
    }
}

impl MultiThreadBase for VmGdbStubTarget {
    fn read_registers(
        &mut self,
        regs: &mut <GdbStubArch as Arch>::Registers,
        tid: Tid,
    ) -> TargetResult<(), Self> {
        let vcpu_id = tid_to_vcpu_id(tid);

        let response = GdbStubCommand::ReadRegisters { vcpu_id }
            .send_and_then_wait(&self.tx)
            .map_err(|_| TargetError::NonFatal)?;

        match response {
            GdbStubCommandResponse::ReadRegisters { registers } => {
                *regs = *registers;

                Ok(())
            }
            GdbStubCommandResponse::Err => {
                error!("Failed to handle ReadRegisters command");
                Err(TargetError::NonFatal)
            }
            _ => {
                error!("Unexpected response to ReadRegisters command");
                Err(TargetError::NonFatal)
            }
        }
    }

    fn write_registers(
        &mut self,
        regs: &<GdbStubArch as Arch>::Registers,
        tid: Tid,
    ) -> TargetResult<(), Self> {
        let vcpu_id = tid_to_vcpu_id(tid);

        let response = GdbStubCommand::WriteRegisters {
            vcpu_id,
            registers: Box::new(regs.clone()),
        }
        .send_and_then_wait(&self.tx)
        .map_err(|_| TargetError::NonFatal)?;

        match response {
            GdbStubCommandResponse::WriteRegisters => Ok(()),
            GdbStubCommandResponse::Err => {
                error!("Failed to handle command");
                Err(TargetError::NonFatal)
            }
            _ => {
                error!("Unexpected response to command");
                Err(TargetError::NonFatal)
            }
        }
    }

    fn read_addrs(
        &mut self,
        start_addr: <GdbStubArch as Arch>::Usize,
        data: &mut [u8],
        tid: Tid,
    ) -> TargetResult<usize, Self> {
        let vcpu_id = tid_to_vcpu_id(tid);

        let response = GdbStubCommand::ReadAddrs {
            gva: start_addr,
            len: data.len(),
            vcpu_id,
        }
        .send_and_then_wait(&self.tx)
        .map_err(|_| TargetError::NonFatal)?;

        match response {
            GdbStubCommandResponse::ReadAddrs { buf } => {
                data[..buf.len()].copy_from_slice(&buf);
                Ok(data.len())
            }
            GdbStubCommandResponse::Err => {
                error!("Failed to handle command");
                Err(TargetError::NonFatal)
            }
            _ => {
                error!("Unexpected response to command");
                Err(TargetError::NonFatal)
            }
        }
    }

    fn write_addrs(
        &mut self,
        start_addr: <GdbStubArch as Arch>::Usize,
        data: &[u8],
        tid: Tid,
    ) -> TargetResult<(), Self> {
        let vcpu_id = tid_to_vcpu_id(tid);

        let response = GdbStubCommand::WriteAddrs {
            gva: start_addr,
            data: data.to_vec(),
            vcpu_id,
        }
        .send_and_then_wait(&self.tx)
        .map_err(|_| TargetError::NonFatal)?;

        match response {
            GdbStubCommandResponse::WriteAddrs => Ok(()),
            GdbStubCommandResponse::Err => {
                error!("Failed to handle command");
                Err(TargetError::NonFatal)
            }
            _ => {
                error!("Unexpected response to command");
                Err(TargetError::NonFatal)
            }
        }
    }

    fn list_active_threads(
        &mut self,
        thread_is_active: &mut dyn FnMut(Tid),
    ) -> Result<(), VmGdbStubError> {
        match GdbStubCommand::ListActiveThreads.send_and_then_wait(&self.tx)? {
            GdbStubCommandResponse::ListActiveThreads(len) => {
                for vcpu_id in 0..len {
                    let tid = vcpu_id_to_tid(vcpu_id)?;
                    thread_is_active(tid);
                }

                Ok(())
            }
            GdbStubCommandResponse::Err => {
                error!("Failed to handle command");
                Err(VmGdbStubError::ListActiveThreadsFailed)
            }
            _ => {
                error!("Unexpected response to command");
                Err(VmGdbStubError::InvalidResponse)
            }
        }
    }

    #[inline(always)]
    fn support_resume(&mut self) -> Option<MultiThreadResumeOps<'_, Self>> {
        Some(self)
    }
}

impl MultiThreadResume for VmGdbStubTarget {
    fn resume(&mut self) -> Result<(), Self::Error> {
        match GdbStubCommand::Resume.send_and_then_wait(&self.tx)? {
            GdbStubCommandResponse::Resume => Ok(()),
            GdbStubCommandResponse::Err => {
                error!("Failed to handle command");
                Err(VmGdbStubError::ResumeFailed)
            }
            _ => {
                error!("Unexpected response to command");
                Err(VmGdbStubError::InvalidResponse)
            }
        }
    }

    fn clear_resume_actions(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_resume_action_continue(
        &mut self,
        _tid: Tid,
        _signal: Option<Signal>,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}

impl Target for VmGdbStubTarget {
    type Arch = GdbStubArch;
    type Error = VmGdbStubError;

    #[inline(always)]
    fn guard_rail_implicit_sw_breakpoints(&self) -> bool {
        true
    }

    fn base_ops(&mut self) -> BaseOps<'_, GdbStubArch, VmGdbStubError> {
        BaseOps::MultiThread(self)
    }
}
