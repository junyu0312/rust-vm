use std::io;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::UnixListener;
use tokio::net::UnixStream;
use tokio::sync::mpsc::Sender;
use vm_core::monitor::MonitorError;

use crate::service::monitor::command::MonitorCommand;
use crate::vmm::Vmm;
use crate::vmm::handler::VmmCommand;

const PATH: &str = "/tmp/vm.sock";

struct MonitorConnection {
    tx: Arc<Sender<VmmCommand>>,
}

impl MonitorConnection {
    fn start(&self, mut stream: UnixStream) {
        let tx = self.tx.clone();

        tokio::spawn({
            async move {
                loop {
                    stream.readable().await?;

                    let mut buf = vec![0u8; 1024];
                    match stream.try_read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let line = match str::from_utf8(&buf[..n]) {
                                Ok(line) => line.trim(),
                                Err(err) => {
                                    stream.write_all(format!("ERR {err}\n").as_bytes()).await?;

                                    continue;
                                }
                            };
                            if line.is_empty() {
                                continue;
                            }

                            let cmd = MonitorCommand(line.to_string());
                            match cmd.send_and_then_wait(&tx).await {
                                Ok(resp) => {
                                    stream.writable().await?;

                                    stream.write_all(resp.0.as_bytes()).await?;
                                }
                                Err(err) => {
                                    stream.write_all(format!("ERR {err}\n").as_bytes()).await?;
                                }
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }

                Ok::<(), MonitorError>(())
            }
        });
    }
}

impl Vmm {
    pub fn listen_for_monitor_client(&self) {
        let tx = self.command_tx.clone();

        tokio::spawn(async move {
            let Ok(listener) = UnixListener::bind(PATH) else {
                return;
            };

            loop {
                let stream = match listener.accept().await {
                    Ok((stream, _)) => stream,
                    Err(_err) => {
                        continue;
                    }
                };

                let monitor_connection = MonitorConnection { tx: tx.clone() };

                monitor_connection.start(stream);
            }
        });
    }
}
