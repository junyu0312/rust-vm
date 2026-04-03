use std::sync::Arc;
use std::sync::Mutex;

use gdbstub::arch::Arch;
use gdbstub::common::Tid;
use gdbstub::target::Target;
use gdbstub::target::TargetResult;
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::base::multithread::MultiThreadBase;
use vm_core::cpu::vcpu_manager::VcpuManager;

use crate::service::gdbstub::GdbStubArch;
use crate::service::gdbstub::error::VmGdbStubError;

fn vcpu_id_to_tid(vcpu_id: usize) -> Result<Tid, VmGdbStubError> {
    Tid::new(vcpu_id + 1).ok_or(VmGdbStubError::InvalidTid)
}

fn tid_to_vcpu_id(tid: Tid) -> usize {
    tid.get() as usize - 1
}

pub struct VmGdbStubTarget {
    vcpu_manager: Arc<Mutex<VcpuManager>>,
}

impl VmGdbStubTarget {
    pub fn new(vcpu_manager: Arc<Mutex<VcpuManager>>) -> VmGdbStubTarget {
        VmGdbStubTarget { vcpu_manager }
    }
}

impl MultiThreadBase for VmGdbStubTarget {
    fn read_registers(
        &mut self,
        _regs: &mut <GdbStubArch as Arch>::Registers,
        _tid: Tid,
    ) -> TargetResult<(), Self> {
        todo!()
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
        todo!()
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
        let vcpu_manager = self.vcpu_manager.lock().unwrap();

        for vcpu_id in 0..vcpu_manager.get_active_vcpus() {
            let tid = vcpu_id_to_tid(vcpu_id)?;
            thread_is_active(tid);
        }

        Ok(())
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
