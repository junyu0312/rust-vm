use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tracing::error;
use vm_core::virtualization::hypervisor::Hypervisor;

use crate::error::Error;
use crate::error::Result;
use crate::vm::Vm;
use crate::vm::config::VmConfig;
use crate::vmm::command::VmmCommand;

pub mod command;

pub struct Vmm {
    hypervisor: Box<dyn Hypervisor>,
    vm: Option<Vm>,
    command_rx: Receiver<VmmCommand>,
    command_tx: Arc<Sender<VmmCommand>>,
}

impl Vmm {
    pub fn new(hypervisor: Box<dyn Hypervisor>) -> Self {
        let (command_tx, command_rx) = mpsc::channel(1024);

        Vmm {
            hypervisor,
            vm: None,
            command_rx,
            command_tx: Arc::new(command_tx),
        }
    }

    pub async fn create_vm_from_config(&mut self, vm_config: VmConfig) -> Result<()> {
        if self.vm.is_some() {
            return Err(Error::VmAlreadyExists);
        }

        let vm =
            Vm::from_config(self.hypervisor.as_ref(), self.command_tx.clone(), vm_config).await?;

        self.vm = Some(vm);

        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        self.vm.as_mut().ok_or(Error::VmNotExists)?.boot().await?;

        if let Err(err) = self.run_monitor().await {
            error!(?err, "monitor error");
        }

        Ok(())
    }
}
