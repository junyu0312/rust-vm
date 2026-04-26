use std::thread::sleep;
use std::time::Duration;

use gdbstub::arch::Arch;
use gdbstub::conn::ConnectionExt;
use gdbstub::stub::MultiThreadStopReason;
use gdbstub::stub::run_blocking::BlockingEventLoop;
use gdbstub::stub::run_blocking::Event;
use gdbstub::stub::run_blocking::WaitForStopReasonError;

use crate::service::gdbstub::GdbStubArch;
use crate::service::gdbstub::error::VmGdbStubError;
use crate::service::gdbstub::target::VmGdbStubTarget;

pub struct VmEventLoop {}

impl BlockingEventLoop for VmEventLoop {
    type Target = VmGdbStubTarget;
    type Connection = Box<dyn ConnectionExt<Error = std::io::Error>>;
    type StopReason = MultiThreadStopReason<<GdbStubArch as Arch>::Usize>;

    fn wait_for_stop_reason(
        _target: &mut VmGdbStubTarget,
        _conn: &mut Box<dyn ConnectionExt<Error = std::io::Error>>,
    ) -> Result<
        Event<MultiThreadStopReason<<GdbStubArch as Arch>::Usize>>,
        WaitForStopReasonError<VmGdbStubError, std::io::Error>,
    > {
        loop {
            sleep(Duration::from_secs(1));
        }
    }

    fn on_interrupt(
        _target: &mut VmGdbStubTarget,
    ) -> Result<Option<MultiThreadStopReason<<GdbStubArch as Arch>::Usize>>, VmGdbStubError> {
        todo!()
    }
}
