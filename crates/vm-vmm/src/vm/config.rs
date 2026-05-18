use std::path::PathBuf;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
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
use vm_core::virtualization::vm::error::VmError;
use vm_device::device::Device;
use vm_mm::allocator::Allocator;
use vm_mm::allocator::std_allocator::StdAllocator;
use vm_mm::manager::MemoryAddressSpace;
use vm_mm::memory_container::MemoryContainer;
use vm_mm::region::MemoryRegion;

use crate::device::InitDevice;
use crate::service::gdbstub::connection::VmGdbStubConnector;
use crate::service::monitor::builder::MonitorServerBuilder;
use crate::vm::PAGE_SIZE;
use crate::vm::Vm;
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
        let mut monitor_server_builder = MonitorServerBuilder::default();

        let vm_instance = hypervisor.create_vm()?;

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
                .map_err(|_| VmError::MemoryRegionOverlap)?;
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

        let vm_exit_handler = Arc::new(VmExitHandler::new(
            device_manager.clone(),
            #[cfg(target_arch = "aarch64")]
            psci,
        ));

        {
            let mut vcpu_manager = vcpu_manager.lock().await;

            for vcpu_id in 0..vm_config.vcpus {
                vcpu_manager.create_vcpu(
                    vcpu_id,
                    memory_address_space.clone(),
                    vm_exit_handler.clone(),
                )?;
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            use vm_bootloader::boot_loader::BootLoader;
            use vm_bootloader::boot_loader::BootLoaderBuilder;
            use vm_bootloader::boot_loader::arch::aarch64::AArch64BootLoader;

            let bootloader = AArch64BootLoader::new(
                vm_config.kernel.clone(),
                vm_config.initramfs.clone(),
                vm_config.cmdline.clone(),
            );

            let mut vcpu_manager = vcpu_manager.lock().await;

            let boot_vcpu = vcpu_manager.get_vcpu_mut(0)?;
            bootloader
                .load(
                    vm_config.memory_size as u64,
                    vm_config.vcpus,
                    boot_vcpu,
                    &memory_address_space,
                    irq_chip.as_ref(),
                    device_manager.mmio_devices(),
                )
                .await?;
        }

        let gdb_stub = vm_config
            .gdb_port
            .map(|port| VmGdbStubConnector::new(vmm_tx, port));

        let vm = Vm {
            vm_config,
            _vm_instance: vm_instance,
            vcpu_manager,
            memory_address_space,
            _irq_chip: irq_chip,
            _device_manager: device_manager,
            gdb_stub,
            monitor_handlers: monitor_server_builder.components,
        };

        Ok(vm)
    }
}
