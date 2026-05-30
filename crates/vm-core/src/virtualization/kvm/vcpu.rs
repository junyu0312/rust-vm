use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use kvm_ioctls::VmFd;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::WeakSender;
use tokio::sync::mpsc::error::TryRecvError;
use tracing::error;

use crate::virtualization::kvm::vcpu::vm_exit::handle_vm_exit;
use crate::virtualization::vcpu::HypervisorVcpu;
use crate::virtualization::vcpu::command::VcpuCommand;
use crate::virtualization::vcpu::command::VcpuCommandRequest;
use crate::virtualization::vcpu::command::VcpuCommandResponse;
use crate::virtualization::vcpu::error::VcpuError;

mod vm_exit;

fn handle_command(
    is_running: &AtomicBool,
    cmd: VcpuCommand,
) -> Result<VcpuCommandResponse, VcpuError> {
    match cmd {
        VcpuCommand::ReadRegisters => todo!(),
        VcpuCommand::WriteRegisters(_) => todo!(),
        VcpuCommand::ReadCoreRegisters => todo!(),
        VcpuCommand::WriteCoreRegisters(_) => todo!(),
        VcpuCommand::Save => todo!(),
        VcpuCommand::Load(_) => todo!(),
        VcpuCommand::TranslateGvaToGpa(_) => todo!(),
        VcpuCommand::Resume => {
            is_running.store(true, Ordering::Release);

            Ok(VcpuCommandResponse::Empty)
        }
    }
}

fn handle_command_and_send_response(is_running: &AtomicBool, request: VcpuCommandRequest) {
    if let Err(_err) = match handle_command(is_running, request.cmd) {
        Ok(resp) => request.response.send(resp),
        Err(err) => request.response.send(VcpuCommandResponse::Err(err)),
    } {
        error!("Failed to send response of vcpu command");
    }
}

pub struct KvmVcpu {
    vcpu_id: u64,
    command_tx: Sender<VcpuCommandRequest>,
}

impl KvmVcpu {
    pub fn new(vm_fd: &VmFd, vcpu_id: u64) -> Result<Self, VcpuError> {
        let mut vcpu_fd = vm_fd.create_vcpu(vcpu_id)?;

        let (command_tx, mut command_rx) = tokio::sync::mpsc::channel(8);
        let is_running = Arc::new(AtomicBool::new(false));

        let _join_handler = {
            let is_running = is_running.clone();

            std::thread::spawn(move || -> Result<(), VcpuError> {
                loop {
                    {
                        match command_rx.try_recv() {
                            Ok(request) => {
                                handle_command_and_send_response(&is_running, request);

                                continue;
                            }
                            Err(TryRecvError::Empty) => (),
                            Err(TryRecvError::Disconnected) => {
                                return Err(VcpuError::VcpuCommandDisconnected);
                            }
                        }
                    }

                    {
                        if is_running.load(Ordering::Acquire) {
                            let vm_exit = vcpu_fd.run()?;

                            handle_vm_exit(vm_exit);
                        }
                    }

                    {
                        command_rx
                            .blocking_recv()
                            .ok_or(VcpuError::VcpuCommandDisconnected)
                            .map(|request| {
                                handle_command_and_send_response(&is_running, request)
                            })?;
                    }
                }
            })
        };

        let vcpu = KvmVcpu {
            vcpu_id,
            command_tx,
        };

        Ok(vcpu)
    }
}

impl HypervisorVcpu for KvmVcpu {
    fn vcpu_id(&self) -> u64 {
        self.vcpu_id
    }

    fn command_tx(&self) -> WeakSender<VcpuCommandRequest> {
        self.command_tx.downgrade()
    }

    fn tick(&self) -> Result<(), VcpuError> {
        todo!()
    }
}
