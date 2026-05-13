use crate::service::gdbstub::command::GdbStubCommandRequest;
use crate::service::monitor::command::MonitorCommandRequest;

pub(crate) mod gdbstub;
pub(crate) mod monitor;

pub enum VmmCommand {
    GdbCommand(GdbStubCommandRequest),
    MonitorCommand(MonitorCommandRequest),
}
