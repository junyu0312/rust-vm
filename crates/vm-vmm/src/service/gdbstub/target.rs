use std::sync::Arc;

use gdbstub::arch::Arch;
use gdbstub::common::Tid;
use gdbstub::target::Target;
use gdbstub::target::TargetError;
use gdbstub::target::TargetResult;
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::base::multithread::MultiThreadBase;
use tokio::sync::mpsc;
use tracing::error;

use crate::service::gdbstub::GdbStubArch;
use crate::service::gdbstub::command::GdbStubCommand;
use crate::service::gdbstub::command::GdbStubCommandResponse;
use crate::service::gdbstub::error::VmGdbStubError;
use crate::vmm::command::VmmCommand;

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
        _regs: &mut <GdbStubArch as Arch>::Registers,
        tid: Tid,
    ) -> TargetResult<(), Self> {
        let vcpu_id = tid_to_vcpu_id(tid);

        let Ok(response) = (GdbStubCommand::ReadRegisters { vcpu_id }).send_and_then_wait(&self.tx)
        else {
            return Err(TargetError::NonFatal);
        };

        match response {
            Ok(GdbStubCommandResponse::ReadRegisters) => {
                todo!();
            }
            Ok(_) => {
                error!("Unexpected response to ReadRegisters command");
                Err(TargetError::NonFatal)
            }
            Err(err) => {
                error!(?err, "Failed to read registers");
                Err(TargetError::NonFatal)
            }
        }
    }

    fn write_registers(
        &mut self,
        _regs: &<GdbStubArch as Arch>::Registers,
        _tid: Tid,
    ) -> TargetResult<(), Self> {
        todo!()
    }

    fn read_addrs(
        &mut self,
        _start_addr: <GdbStubArch as Arch>::Usize,
        _data: &mut [u8],
        _tid: Tid,
    ) -> TargetResult<usize, Self> {
        eprintln!(
            "read_addrs: start_addr={:#x}, len={}, tid={}",
            _start_addr,
            _data.len(),
            _tid.get()
        );
        Err(TargetError::NonFatal)
    }

    fn write_addrs(
        &mut self,
        _start_addr: <GdbStubArch as Arch>::Usize,
        _data: &[u8],
        _tid: Tid,
    ) -> TargetResult<(), Self> {
        todo!()
    }

    fn list_active_threads(
        &mut self,
        thread_is_active: &mut dyn FnMut(Tid),
    ) -> Result<(), VmGdbStubError> {
        match GdbStubCommand::ListActiveThreads.send_and_then_wait(&self.tx)? {
            Ok(GdbStubCommandResponse::ListActiveThreads(len)) => {
                for vcpu_id in 0..len {
                    let tid = vcpu_id_to_tid(vcpu_id)?;
                    thread_is_active(tid);
                }

                Ok(())
            }
            Ok(_) => Err(VmGdbStubError::InvalidResponse),
            Err(err) => {
                error!(?err, "Failed to list active threads");
                Err(VmGdbStubError::ListActiveThreadsFailed)
            }
        }
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
