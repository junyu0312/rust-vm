use std::io;
use std::net::TcpListener;
use std::sync::Arc;
use std::thread::JoinHandle;

use gdbstub::arch::Arch;
use gdbstub::stub::DisconnectReason;
use gdbstub::stub::GdbStub;
use vm_mm::manager::MemoryAddressSpace;
use vm_mm::memory_container::MemoryContainer;

use crate::debug::gdbstub::event_loop::VmEventLoop;
use crate::debug::gdbstub::target::VmTarget;

mod event_loop;
mod target;

#[derive(Debug, thiserror::Error)]
pub enum GdbStubError {
    #[error("{0}")]
    IO(#[from] io::Error),
}

pub struct GdbStubBuilder<C> {
    mm: Arc<MemoryAddressSpace<C>>,
    port: u16,
}

impl<C> GdbStubBuilder<C>
where
    C: MemoryContainer,
{
    pub fn new(mm: Arc<MemoryAddressSpace<C>>, port: u16) -> Self {
        GdbStubBuilder { mm, port }
    }

    pub fn wait_and_then_run<A>(&self) -> Result<JoinHandle<DisconnectReason>, GdbStubError>
    where
        A: Arch,
    {
        let sockaddr = format!("localhost:{}", self.port);
        eprintln!("Waiting for a GDB connection on {:?}...", sockaddr);
        let sock = TcpListener::bind(sockaddr)?;
        let (stream, addr) = sock.accept()?;

        eprintln!("Debugger connected from {}", addr);

        let mm = self.mm.clone();

        Ok(std::thread::spawn(move || {
            let mut target = VmTarget::<A, C>::new(mm);
            let gdb_stub = GdbStub::new(Box::new(stream) as _);

            match gdb_stub.run_blocking::<VmEventLoop<A, C>>(&mut target) {
                Ok(disconnect_reason) => match disconnect_reason {
                    DisconnectReason::TargetExited(_) => todo!(),
                    DisconnectReason::TargetTerminated(_signal) => todo!(),
                    DisconnectReason::Disconnect => todo!(),
                    DisconnectReason::Kill => todo!(),
                },
                Err(err) if err.is_connection_error() => todo!(),
                Err(err) if err.is_target_error() => todo!(),
                Err(err) => {
                    eprintln!("{err}");
                    panic!()
                }
            }
        }))
    }
}
