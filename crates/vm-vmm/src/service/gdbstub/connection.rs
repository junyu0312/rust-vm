use std::net::TcpListener;
use std::sync::Arc;

use gdbstub::stub::DisconnectReason;
use gdbstub::stub::GdbStub;
use tokio::sync::mpsc;
use tracing::error;

use crate::service::gdbstub::error::VmGdbStubError;
use crate::service::gdbstub::event_loop::VmEventLoop;
use crate::service::gdbstub::target::VmGdbStubTarget;
use crate::vmm::command::VmmCommand;

pub struct VmGdbStubConnector {
    tx: Arc<mpsc::Sender<VmmCommand>>,
    port: u16,
}

impl VmGdbStubConnector {
    pub fn new(tx: Arc<mpsc::Sender<VmmCommand>>, port: u16) -> VmGdbStubConnector {
        VmGdbStubConnector { tx, port }
    }
}

impl VmGdbStubConnector {
    pub fn wait_for_connection(&self) -> Result<(), VmGdbStubError> {
        let sockaddr = format!("localhost:{}", self.port);
        let listener = TcpListener::bind(&sockaddr)?;

        eprintln!("Waiting for a GDB connection on {:?}...", sockaddr);

        let (stream, addr) = listener.accept()?;

        eprintln!("{} connected", addr);

        let tx = self.tx.clone();

        std::thread::spawn(move || {
            let connection = Box::new(stream) as _;
            let gdbstub = GdbStub::new(connection);
            let mut target = VmGdbStubTarget::new(tx);

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
