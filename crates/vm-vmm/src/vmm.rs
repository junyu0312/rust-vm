use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use vm_bootloader::boot_loader::BootLoader;
#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::firmware::psci::psci_0_2::Psci02;
#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::layout::MMIO_LEN;
#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::layout::MMIO_START;
#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::layout::RAM_BASE;
#[cfg(target_arch = "x86_64")]
use vm_core::arch::x86_64::layout::MMIO_LEN;
#[cfg(target_arch = "x86_64")]
use vm_core::arch::x86_64::layout::MMIO_START;
#[cfg(target_arch = "x86_64")]
use vm_core::arch::x86_64::layout::RAM_BASE;
use vm_core::cpu::vcpu_manager::VcpuManager;
use vm_core::device::mmio::layout::MmioLayout;
use vm_core::device_manager::DeviceManager;
use vm_core::virtualization::hypervisor::Hypervisor;
use vm_core::virtualization::vm::SetUserMemoryRegionFlags;
use vm_device::device::Device;
use vm_mm::allocator::Allocator;
use vm_mm::allocator::std_allocator::StdAllocator;
use vm_mm::manager::MemoryAddressSpace;
use vm_mm::memory_container::MemoryContainer;
use vm_mm::region::MemoryRegion;

use crate::device::InitDevice;
use crate::error::Error;
use crate::error::Result;
use crate::service::gdbstub::connection::VmGdbStubConnector;
use crate::service::monitor::MonitorServerBuilder;
use crate::vm::Vm;
use crate::vm::config::VmConfig;
use crate::vm::vm_exit_handler::VmExitHandler;
use crate::vmm::command::VmmCommand;

const PAGE_SIZE: usize = 4 << 10;

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

        let mut monitor_server_builder = MonitorServerBuilder::default();

        let vm_instance = self.hypervisor.create_vm()?;

        let mut memory_address_space = MemoryAddressSpace::default();
        {
            let memory_region = StdAllocator.alloc(vm_config.memory_size, Some(PAGE_SIZE))?;

            vm_instance.set_user_memory_region(
                memory_region.hva() as _,
                RAM_BASE,
                vm_config.memory_size,
                SetUserMemoryRegionFlags::ReadWriteExec,
            )?;
            memory_address_space
                .try_insert(MemoryRegion::new(RAM_BASE, Box::new(memory_region)))
                .map_err(|_| Error::InitMemory("Failed to initialize memory".to_string()))?;
        }

        let memory_address_space = Arc::new(memory_address_space);

        let irq_chip = if !vm_config.devices.iter().any(Device::is_irq_chip) {
            vm_instance.create_irq_chip()?
        } else {
            todo!()
        };

        let mut device_manager = DeviceManager::new(MmioLayout::new(MMIO_START, MMIO_LEN));
        device_manager.init_devices(
            &mut monitor_server_builder,
            memory_address_space.clone(),
            &vm_config.devices,
            irq_chip.clone(),
        )?;
        let device_manager = Arc::new(device_manager);

        let vcpu_manager = Arc::new(Mutex::new(VcpuManager::new(vm_instance.clone())));

        #[cfg(target_arch = "aarch64")]
        let psci = Psci02 {
            vcpu_manager: vcpu_manager.clone(),
        };

        let vm_exit_handler = Arc::new(VmExitHandler {
            device_manager: device_manager.clone(),
            #[cfg(target_arch = "aarch64")]
            psci,
        });

        {
            let mut vcpu_manager = vcpu_manager.lock().await;

            for vcpu_id in 0..vm_config.vcpus {
                vcpu_manager.create_vcpu(vcpu_id, vm_exit_handler.clone())?;
            }
        }

        let mut start_pc = 0;
        #[cfg(target_arch = "aarch64")]
        {
            use vm_bootloader::boot_loader::BootLoaderBuilder;
            use vm_bootloader::boot_loader::arch::aarch64::AArch64BootLoader;

            let bootloader = AArch64BootLoader::new(
                vm_config.kernel.clone(),
                vm_config.initramfs.clone(),
                vm_config.cmdline.clone(),
            );

            start_pc = bootloader.load(
                vm_config.memory_size as u64,
                vm_config.vcpus,
                &memory_address_space,
                irq_chip.as_ref(),
                device_manager.mmio_devices(),
            )?;
        }

        let vm = Vm {
            _vm_instance: vm_instance,
            vcpu_manager,
            memory_address_space,
            irq_chip,
            device_manager,
            gdb_stub: vm_config
                .gdb_port
                .map(|port| VmGdbStubConnector::new(self.command_tx.clone(), port)),
            monitor: monitor_server_builder.build(),
            vm_config,
            start_pc,
        };

        self.vm = Some(vm);

        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        self.vm.as_mut().ok_or(Error::VmNotExists)?.boot().await?;

        self.run_monitor().await?;

        Ok(())
    }
}
