use std::marker::PhantomData;
use std::sync::Arc;

use gdbstub::arch::Arch;
use gdbstub::common::Tid;
use gdbstub::target::Target;
use gdbstub::target::TargetResult;
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::base::multithread::MultiThreadBase;
use vm_mm::manager::MemoryAddressSpace;

fn tid_to_vcpu_id(tid: Tid) -> usize {
    tid.get() - 1
}

#[derive(Debug, thiserror::Error)]
pub enum VmTargetError {}

pub struct VmTarget<A, C> {
    mm: Arc<MemoryAddressSpace<C>>,
    _mark: PhantomData<A>,
}

impl<A, C> VmTarget<A, C> {
    pub fn new(mm: Arc<MemoryAddressSpace<C>>) -> Self {
        VmTarget {
            mm,
            _mark: PhantomData,
        }
    }
}

impl<A, C> Target for VmTarget<A, C>
where
    A: Arch,
{
    type Arch = A;

    type Error = VmTargetError;

    #[inline(always)]
    fn base_ops(&mut self) -> BaseOps<'_, Self::Arch, Self::Error> {
        BaseOps::MultiThread(self)
    }
}

impl<A, C> MultiThreadBase for VmTarget<A, C>
where
    A: Arch,
{
    fn read_registers(&mut self, regs: &mut A::Registers, tid: Tid) -> TargetResult<(), Self> {
        let vcpu_id = tid_to_vcpu_id(tid);
        todo!()
    }

    fn write_registers(&mut self, regs: &A::Registers, tid: Tid) -> TargetResult<(), Self> {
        let vcpu_id = tid_to_vcpu_id(tid);
        todo!()
    }

    fn read_addrs(
        &mut self,
        start_addr: A::Usize,
        data: &mut [u8],
        tid: Tid,
    ) -> TargetResult<usize, Self> {
        let vcpu_id = tid_to_vcpu_id(tid);
        todo!()
    }

    fn write_addrs(
        &mut self,
        start_addr: A::Usize,
        data: &[u8],
        tid: Tid,
    ) -> TargetResult<(), Self> {
        let vcpu_id = tid_to_vcpu_id(tid);
        todo!()
    }

    fn list_active_threads(
        &mut self,
        thread_is_active: &mut dyn FnMut(Tid),
    ) -> Result<(), Self::Error> {
        todo!()
    }
}
