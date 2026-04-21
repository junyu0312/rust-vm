use tokio::sync::oneshot;

use crate::arch::registers::ArchCoreRegisters;
use crate::arch::registers::ArchRegisters;

pub enum VcpuCommand {
    ReadRegisters,
    WriteRegisters(ArchRegisters),
    ReadCoreRegisters,
    WriteCoreRegisters(ArchCoreRegisters),
    TranslateGvaToGpa(u64),
    Resume,
    Pause,
}

pub enum VcpuCommandResponse {
    Empty,
    CoreRegisters(Box<ArchCoreRegisters>),
    Registers(Box<ArchRegisters>),
    TranslateGvaToGpa(u64),
}

pub struct VcpuCommandRequest {
    pub cmd: VcpuCommand,
    pub response: oneshot::Sender<VcpuCommandResponse>,
}

impl VcpuCommandRequest {
    pub fn new(cmd: VcpuCommand) -> (Self, oneshot::Receiver<VcpuCommandResponse>) {
        let (tx, rx) = oneshot::channel();

        (VcpuCommandRequest { cmd, response: tx }, rx)
    }
}
