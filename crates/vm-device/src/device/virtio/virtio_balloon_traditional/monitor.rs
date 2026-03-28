use std::sync::Arc;
use std::sync::Mutex;

use async_trait::async_trait;
use serde::Serialize;
use vm_core::monitor::Error;
use vm_core::monitor::MonitorCommand;
use vm_virtio::transport::VirtioDev;

use crate::device::virtio::virtio_balloon_traditional::device::VirtioBalloonApi;
use crate::device::virtio::virtio_balloon_traditional::device::VirtioBalloonTranditional;

#[derive(Serialize)]
pub struct BalloonInfo {
    actual: u32,
    num_pages: u32,
}

pub struct VirtioBalloonMonitor {
    device: Arc<Mutex<VirtioDev<VirtioBalloonTranditional>>>,
}

impl VirtioBalloonMonitor {
    pub fn new(device: Arc<Mutex<VirtioDev<VirtioBalloonTranditional>>>) -> Self {
        VirtioBalloonMonitor { device }
    }
}

#[async_trait]
impl MonitorCommand for VirtioBalloonMonitor {
    async fn handle_command(&self, subcommands: &[&str]) -> Result<String, Error> {
        match *subcommands {
            ["info"] => {
                let dev = self.device.lock().unwrap();
                Ok(serde_json::to_string_pretty(&BalloonInfo {
                    actual: dev.device.cfg.actual,
                    num_pages: dev.device.cfg.num_pages,
                })?)
            }
            ["update_num_pages", num_pages] => {
                let num_pages = num_pages.parse().map_err(|_err| {
                    Error::Error(format!("failed to parse num_pages: {num_pages}"))
                })?;

                let mut dev = self.device.lock().unwrap();
                dev.update_num_pages(num_pages);

                Ok(num_pages.to_string())
            }
            _ => Err(Error::UnknownSubcommand(
                subcommands.iter().map(|s| s.to_string()).collect(),
            )),
        }
    }
}
