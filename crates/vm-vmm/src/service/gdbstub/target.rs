use gdbstub::arch::Arch;
use gdbstub::common::Tid;
use gdbstub::target::Target;
use gdbstub::target::TargetResult;
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::base::multithread::MultiThreadBase;

use crate::service::gdbstub::GdbStubArch;
use crate::service::gdbstub::error::VmGdbStubError;

pub struct VmGdbStubTarget {}

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
        _thread_is_active: &mut dyn FnMut(Tid),
    ) -> Result<(), VmGdbStubError> {
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
