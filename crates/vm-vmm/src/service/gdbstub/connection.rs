use std::net::TcpListener;
use std::sync::Arc;
use std::sync::Mutex;

use gdbstub::stub::DisconnectReason;
use gdbstub::stub::GdbStub;
use tracing::error;
use vm_core::cpu::vcpu_manager::VcpuManager;

use crate::service::gdbstub::error::VmGdbStubError;
use crate::service::gdbstub::event_loop::VmEventLoop;
use crate::service::gdbstub::target::VmGdbStubTarget;

pub struct VmGdbStubConnector {
    vcpu_manager: Arc<Mutex<VcpuManager>>,
    port: u16,
}

impl VmGdbStubConnector {
    pub fn new(vcpu_manager: Arc<Mutex<VcpuManager>>, port: u16) -> VmGdbStubConnector {
        VmGdbStubConnector { vcpu_manager, port }
    }
}

impl VmGdbStubConnector {
    pub fn wait_for_connection(&self) -> Result<(), VmGdbStubError> {
        let sockaddr = format!("localhost:{}", self.port);
        let listener = TcpListener::bind(&sockaddr)?;

        eprintln!("Waiting for a GDB connection on {:?}...", sockaddr);

        let (stream, addr) = listener.accept()?;

        eprintln!("{} connected", addr);

        let vcpu_manager = self.vcpu_manager.clone();

        std::thread::spawn(move || {
            let connection = Box::new(stream) as _;
            let gdbstub = GdbStub::new(connection);
            let mut target = VmGdbStubTarget::new(vcpu_manager);

            match gdbstub.run_blocking::<VmEventLoop>(&mut target) {
                Ok(disconnect_reason) => match disconnect_reason {
                    DisconnectReason::TargetExited(_) => todo!(),
                    DisconnectReason::TargetTerminated(_signal) => todo!(),
                    DisconnectReason::Disconnect => todo!(),
                    DisconnectReason::Kill => todo!(),
                },
                Err(err) => {
                    error!(?err);
                }
            }
        });

        Ok(())
    }
}
