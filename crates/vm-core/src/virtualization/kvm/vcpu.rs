use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

#[cfg(target_arch = "x86_64")]
use kvm_bindings::CpuId;
use kvm_ioctls::VcpuFd;
use kvm_ioctls::VmFd;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::WeakSender;
use tokio::sync::mpsc::error::TryRecvError;
use tracing::error;
use vm_mm::manager::MemoryAddressSpace;

use crate::cpu::vm_exit::VmExit;
use crate::virtualization::kvm::vcpu::vm_exit::VmExitResult;
use crate::virtualization::kvm::vcpu::vm_exit::handle_vm_exit;
use crate::virtualization::vcpu::HypervisorVcpu;
use crate::virtualization::vcpu::command::VcpuCommandRequest;
use crate::virtualization::vcpu::command::VcpuCommandResponse;
use crate::virtualization::vcpu::error::VcpuError;

mod vm_exit;

pub struct KvmVcpuInternal<'a> {
    pub vcpu_fd: &'a VcpuFd,
}

impl<'a> KvmVcpuInternal<'a> {
    fn handle_command_and_send_response(
        &mut self,
        is_running: &AtomicBool,
        request: VcpuCommandRequest,
    ) {
        if let Err(_err) = match self.handle_command(is_running, request.cmd) {
            Ok(resp) => request.response.send(resp),
            Err(err) => request.response.send(VcpuCommandResponse::Err(err)),
        } {
            error!("Failed to send response of vcpu command");
        }
    }
}

pub struct KvmVcpu {
    vcpu_id: u64,
    command_tx: Sender<VcpuCommandRequest>,
}

impl KvmVcpu {
    pub fn new(
        vm_fd: &VmFd,
        vcpu_id: u64,
        #[cfg(target_arch = "x86_64")] supported_cpuid: &CpuId,
        vm_exit_handler: Arc<dyn VmExit>,
        _mm: Arc<MemoryAddressSpace>,
    ) -> Result<Self, VcpuError> {
        let mut vcpu_fd = vm_fd.create_vcpu(vcpu_id)?;
        #[cfg(target_arch = "x86_64")]
        vcpu_fd.set_cpuid2(supported_cpuid)?;

        let (command_tx, mut command_rx) = mpsc::channel(8);
        let is_running = Arc::new(AtomicBool::new(false));

        let _join_handler = {
            let is_running = is_running.clone();

            std::thread::spawn(move || -> Result<(), VcpuError> {
                loop {
                    {
                        match command_rx.try_recv() {
                            Ok(request) => {
                                let mut vcpu = KvmVcpuInternal { vcpu_fd: &vcpu_fd };

                                vcpu.handle_command_and_send_response(&is_running, request);

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
                            // vcpu_fd
                            //     .set_guest_debug(&kvm_guest_debug {
                            //         control: KVM_GUESTDBG_ENABLE | KVM_GUESTDBG_SINGLESTEP,
                            //         ..Default::default()
                            //     })
                            //     .unwrap();

                            let vm_exit = vcpu_fd.run()?;

                            match handle_vm_exit(vm_exit, vm_exit_handler.as_ref()) {
                                Ok(result) => match result {
                                    VmExitResult::Ok => continue,
                                },
                                Err(err) => {
                                    error!(?err);

                                    panic!()
                                }
                            }
                        }
                    }

                    {
                        command_rx
                            .blocking_recv()
                            .ok_or(VcpuError::VcpuCommandDisconnected)
                            .map(|request| {
                                let mut vcpu = KvmVcpuInternal { vcpu_fd: &vcpu_fd };

                                vcpu.handle_command_and_send_response(&is_running, request)
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
