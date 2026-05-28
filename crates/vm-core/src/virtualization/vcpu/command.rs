use tokio::sync::oneshot;

use crate::arch::registers::ArchCoreRegisters;
use crate::arch::registers::ArchRegisters;
use crate::virtualization::vcpu::error::VcpuError;

pub enum VcpuCommand {
    ReadRegisters,
    WriteRegisters(Box<ArchRegisters>),
    ReadCoreRegisters,
    WriteCoreRegisters(Box<ArchCoreRegisters>),
    Save,
    Load(Vec<u8>),
    TranslateGvaToGpa(u64),
    Resume,
}

pub enum VcpuCommandResponse {
    Empty,
    CoreRegisters(Box<ArchCoreRegisters>),
    Registers(Box<ArchRegisters>),
    Save(Vec<u8>),
    TranslateGvaToGpa(Option<u64>),
    Err(VcpuError),
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
