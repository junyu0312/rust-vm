use std::io;
use std::net::TcpListener;
use std::net::TcpStream;

pub struct GdbStub {
    port: u16,
}

impl GdbStub {
    pub fn new(port: u16) -> Self {
        GdbStub { port }
    }

    pub fn wait_for_connection(&self) -> io::Result<TcpStream> {
        let sockaddr = format!("localhost:{}", self.port);
        eprintln!("Waiting for a GDB connection on {:?}...", sockaddr);
        let sock = TcpListener::bind(sockaddr)?;
        let (stream, addr) = sock.accept()?;

        eprintln!("Debugger connected from {}", addr);

        Ok(stream)
    }
}
