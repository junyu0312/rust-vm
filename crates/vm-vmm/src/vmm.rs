use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use vm_core::virtualization::hypervisor::Hypervisor;

use crate::vm::Vm;
use crate::vm::config::VmConfig;
use crate::vmm::error::VmmError;
use crate::vmm::handler::VmmCommand;

pub mod error;

pub(crate) mod handler;

mod service;

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

    pub fn try_get_vm(&self) -> Result<&Vm, VmmError> {
        self.vm.as_ref().ok_or(VmmError::VmNotExists)
    }

    pub fn try_get_vm_mut(&mut self) -> Result<&mut Vm, VmmError> {
        self.vm.as_mut().ok_or(VmmError::VmNotExists)
    }

    pub async fn create_vm_from_config(&mut self, vm_config: VmConfig) -> Result<(), VmmError> {
        if self.vm.is_some() {
            return Err(VmmError::VmAlreadyExists);
        }

        let vm =
            Vm::from_config(self.hypervisor.as_ref(), self.command_tx.clone(), vm_config).await?;

        self.vm = Some(vm);

        Ok(())
    }

    pub async fn pause(&mut self) -> Result<(), VmmError> {
        let vm = self.try_get_vm_mut()?;

        vm.pause().await?;

        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), VmmError> {
        self.vm
            .as_mut()
            .ok_or(VmmError::VmNotExists)?
            .boot()
            .await?;

        self.run_monitor().await;

        Ok(())
    }
}
