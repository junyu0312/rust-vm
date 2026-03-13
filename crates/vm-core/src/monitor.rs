use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixListener;
use tokio::net::UnixStream;

const PATH: &str = "/tmp/vm.sock";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Stream(#[from] std::io::Error),

    #[error("{0}")]
    CommandHandlerConflicat(String),

    #[error("Unknown cmd {0}")]
    UnknownCmd(String),
}

#[async_trait]
pub trait MonitorCommand: Send + Sync {
    async fn handle_command(&self, subcommands: &[&str]) -> Result<String, Error>;
}

struct MonitorConnection {
    components: Arc<HashMap<String, Box<dyn MonitorCommand>>>,
}

impl MonitorConnection {
    fn start(&self, mut stream: UnixStream) {
        tokio::spawn({
            let components = self.components.clone();

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

                            println!("line: {line}");
                            let mut tokens = line.split_whitespace();
                            let command = match tokens.next() {
                                Some(f) => f,
                                None => continue,
                            };
                            let subcommands: Vec<&str> = tokens.collect();

                            match components.get(command) {
                                Some(handler) => match handler.handle_command(&subcommands).await {
                                    Ok(resp) => {
                                        stream.writable().await?;

                                        stream.write_all(resp.as_bytes()).await?;
                                    }
                                    Err(e) => {
                                        stream.write_all(format!("ERR {e}\n").as_bytes()).await?;
                                    }
                                },
                                None => {
                                    println!("write aaa");
                                    stream
                                        .write_all(
                                            format!("ERR unknown command {command}\n").as_bytes(),
                                        )
                                        .await?;
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

                Ok::<(), Error>(())
            }
        });
    }
}

pub struct MonitorServer {
    components: Arc<HashMap<String, Box<dyn MonitorCommand>>>,
}

impl MonitorServer {
    pub fn start(&self) {
        let components = self.components.clone();

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

                let monitor_connection = MonitorConnection {
                    components: components.clone(),
                };

                monitor_connection.start(stream);
            }
        });
    }
}

#[derive(Default)]
pub struct MonitorServerBuilder {
    components: HashMap<String, Box<dyn MonitorCommand>>,
}

impl MonitorServerBuilder {
    pub fn register_command_handler(
        &mut self,
        name: &str,
        handler: Box<dyn MonitorCommand>,
    ) -> Result<(), Error> {
        let name = name.to_string();

        if self.components.contains_key(&name) {
            return Err(Error::CommandHandlerConflicat(name));
        }

        self.components.insert(name, handler);

        Ok(())
    }

    pub fn build(self) -> MonitorServer {
        MonitorServer {
            components: self.components.into(),
        }
    }
}
