use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use tokio::sync::Mutex;
use vm_core::monitor::MonitorCommandOps;
use vm_core::monitor::MonitorError;
use vm_virtio::device::virtqueue::VirtioConfigurationChangeNotifier;
use vm_virtio::types::device::balloon_tranditional::VirtioBalloonTranditionalConfig;

#[derive(Serialize)]
pub struct BalloonInfo {
    actual: u32,
    num_pages: u32,
}

pub struct VirtioBalloonMonitor {
    config: Arc<Mutex<VirtioBalloonTranditionalConfig>>,
    configuration_change_notifier: Arc<dyn VirtioConfigurationChangeNotifier>,
}

impl VirtioBalloonMonitor {
    pub fn new(
        config: Arc<Mutex<VirtioBalloonTranditionalConfig>>,
        configuration_change_notifier: Arc<dyn VirtioConfigurationChangeNotifier>,
    ) -> Self {
        VirtioBalloonMonitor {
            config,
            configuration_change_notifier,
        }
    }
}

#[async_trait]
impl MonitorCommandOps for VirtioBalloonMonitor {
    async fn handle_command(&self, subcommands: &[&str]) -> Result<String, MonitorError> {
        let mut config = self.config.lock().await;

        match *subcommands {
            ["info"] => Ok(serde_json::to_string_pretty(&BalloonInfo {
                actual: config.actual,
                num_pages: config.num_pages,
            })?),
            ["update_num_pages", num_pages] => {
                let num_pages = num_pages.parse().map_err(|_err| {
                    MonitorError::Error(format!("failed to parse num_pages: {num_pages}"))
                })?;

                config.num_pages = num_pages;
                self.configuration_change_notifier
                    .update_config_generation();

                Ok(num_pages.to_string())
            }
            _ => Err(MonitorError::UnknownSubcommand(
                subcommands.iter().map(|s| s.to_string()).collect(),
            )),
        }
    }
}
