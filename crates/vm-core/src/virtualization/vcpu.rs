use tokio::sync::mpsc::WeakSender;

use crate::cpu::error::VcpuError;
use crate::virtualization::vcpu::command::VcpuCommandRequest;

pub(crate) mod command;

pub trait HypervisorVcpu: Send {
    fn vcpu_id(&self) -> usize;

    fn command_tx(&self) -> WeakSender<VcpuCommandRequest>;

    fn tick(&mut self) -> Result<(), VcpuError>;
}
