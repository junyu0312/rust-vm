use std::io;
use std::marker::PhantomData;

use gdbstub::arch::Arch;
use gdbstub::conn::Connection;
use gdbstub::conn::ConnectionExt;
use gdbstub::stub::MultiThreadStopReason;
use gdbstub::stub::run_blocking::BlockingEventLoop;
use gdbstub::stub::run_blocking::Event;
use gdbstub::stub::run_blocking::WaitForStopReasonError;

use crate::debug::gdbstub::target::VmTarget;
use crate::debug::gdbstub::target::VmTargetError;

pub struct VmEventLoop<A, C> {
    _mark: PhantomData<(A, C)>,
}

impl<A, C> BlockingEventLoop for VmEventLoop<A, C>
where
    A: Arch,
{
    type Target = VmTarget<A, C>;

    type Connection = Box<dyn ConnectionExt<Error = io::Error>>;

    type StopReason = MultiThreadStopReason<A::Usize>;

    fn wait_for_stop_reason(
        target: &mut VmTarget<A, C>,
        conn: &mut Self::Connection,
    ) -> Result<
        Event<Self::StopReason>,
        WaitForStopReasonError<VmTargetError, <Self::Connection as Connection>::Error>,
    > {
        todo!()
    }

    fn on_interrupt(
        target: &mut VmTarget<A, C>,
    ) -> Result<Option<Self::StopReason>, VmTargetError> {
        todo!()
    }
}
