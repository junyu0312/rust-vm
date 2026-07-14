use std::path::PathBuf;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::firmware::psci::psci_0_2::Psci02;
#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::layout::RAM_BASE;
use vm_core::arch::irq::InterruptController;
#[cfg(target_arch = "x86_64")]
use vm_core::arch::x86_64::layout::RAM_BASE;
use vm_core::cpu::vcpu_manager::VcpuManager;
use vm_core::virtualization::hypervisor::Hypervisor;
use vm_core::virtualization::vm::SetUserMemoryRegionFlags;
use vm_core::virtualization::vm::error::VmError;
use vm_core::virtualization::vm::state::VmState;
use vm_device::device::Device;
use vm_mm::allocator::Allocator;
use vm_mm::allocator::std_allocator::StdAllocator;
use vm_mm::manager::MemoryAddressSpace;
use vm_mm::region::MemoryRegion;
use vm_utils::range_allocator::RangeAllocator;

#[cfg(target_arch = "aarch64")]
use crate::bootloader::aarch64::install_bootloader;
#[cfg(target_arch = "x86_64")]
use crate::bootloader::x86_64::install_bootloader;
use crate::service::gdbstub::connection::VmGdbStubConnector;
use crate::service::monitor::builder::MonitorServerBuilder;
use crate::vm::PAGE_SIZE;
use crate::vm::Vm;
use crate::vm::device_builder::DeviceManagerBuilder;
use crate::vm::vm_exit_handler::VmExitHandler;
use crate::vmm::error::VmmError;
use crate::vmm::handler::VmmCommand;

#[derive(Clone, Serialize, Deserialize)]
pub struct VmConfig {
    pub memory_size: usize,
    pub vcpus: usize,
    pub devices: Vec<Device>,
    pub gdb_port: Option<u16>,
    pub kernel: PathBuf,
    pub initramfs: Option<PathBuf>,
    pub cmdline: Option<String>,
}

impl Vm {
    pub async fn from_config(
        hypervisor: &dyn Hypervisor,
        vmm_tx: Arc<mpsc::Sender<VmmCommand>>,
        vm_config: VmConfig,
    ) -> Result<Self, VmmError> {
        let mut ram_allocator = RangeAllocator::<u64>::default();

        let mut monitor_server_builder = MonitorServerBuilder::default();

        let vm_instance = hypervisor.create_vm()?;

        let mut memory_address_space = MemoryAddressSpace::default();
        {
            let memory_region = StdAllocator.alloc(vm_config.memory_size, Some(PAGE_SIZE))?;

            memory_address_space
                .try_insert(MemoryRegion::new(RAM_BASE, Box::new(memory_region)))
                .map_err(|_| VmError::MemoryRegionOverlap)?;

            for region in memory_address_space.regions().values() {
                ram_allocator.insert(region.gpa, region.len()).unwrap();
                vm_instance.set_user_memory_region(
                    region.hva() as u64,
                    region.gpa,
                    region.len(),
                    SetUserMemoryRegionFlags::ReadWriteExec,
                )?;
            }
        }

        let memory_address_space = Arc::new(memory_address_space);

        let irq_chip: Arc<dyn InterruptController> =
            if !vm_config.devices.iter().any(Device::is_irq_chip) {
                Arc::from(vm_instance.create_irq_chip()?)
            } else {
                todo!()
            };

        let device_manager = {
            let device_manager = DeviceManagerBuilder::new(
                vm_instance.clone(),
                irq_chip.clone(),
                vm_instance.create_irq_manager()?,
                memory_address_space.clone(),
                &mut monitor_server_builder,
            )?
            .build(&vm_config.devices)?;

            Arc::new(device_manager)
        };

        let vcpu_manager = Arc::new(Mutex::new(VcpuManager::new(vm_instance.clone())));

        #[cfg(target_arch = "aarch64")]
        let psci = Psci02 {
            vcpu_manager: vcpu_manager.clone(),
        };

        let vm_exit_handler = Arc::new(VmExitHandler::new(
            device_manager.clone(),
            #[cfg(target_arch = "aarch64")]
            psci,
        ));

        {
            let mut vcpu_manager = vcpu_manager.lock().await;

            for vcpu_id in 0..vm_config.vcpus {
                vcpu_manager.create_vcpu(
                    vcpu_id as u64,
                    memory_address_space.clone(),
                    vm_exit_handler.clone(),
                    false,
                )?;
            }
        }

        install_bootloader(
            &vm_config,
            &vcpu_manager,
            &mut ram_allocator,
            &memory_address_space,
            irq_chip.as_ref(),
            device_manager.as_ref(),
        )
        .await?;

        let gdb_stub = vm_config
            .gdb_port
            .map(|port| VmGdbStubConnector::new(vmm_tx, port));

        let vm = Vm {
            vm_config,
            vm_instance,
            vm_state: VmState::Created,
            vcpu_manager,
            memory_address_space,
            irq_chip,
            device_manager,
            gdb_stub,
            monitor_handlers: monitor_server_builder.components,
        };

        Ok(vm)
    }
}
