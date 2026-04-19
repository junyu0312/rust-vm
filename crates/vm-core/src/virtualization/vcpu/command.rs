use tokio::sync::oneshot;

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::register::AArch64Registers as ArchRegisters;
#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::register::X86_64Registers as ArchRegisters;

pub enum VcpuCommand {
    ReadRegisters,
    WriteRegisters(ArchRegisters),
    Pause,
    Resume,
}

pub enum VcpuCommandResponse {
    Empty,
    Registers(ArchRegisters),
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
