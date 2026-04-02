use std::sync::Arc;
use std::sync::Mutex;

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
use vm_core::debug::gdbstub::GdbStub;
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
use crate::service::monitor::MonitorServerBuilder;
use crate::vm::Vm;
use crate::vm::config::VmConfig;
use crate::vm::vm_exit_handler::VmExitHandler;

const PAGE_SIZE: usize = 4 << 10;

pub struct Vmm {
    hypervisor: Box<dyn Hypervisor>,
    vm: Option<Vm>,
}

impl Vmm {
    pub fn new(hypervisor: Box<dyn Hypervisor>) -> Self {
        Vmm {
            hypervisor,
            vm: None,
        }
    }

    pub fn create_vm_from_config(&mut self, vm_config: VmConfig) -> Result<()> {
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

        for vcpu_id in 0..vm_config.vcpus {
            vcpu_manager
                .lock()
                .unwrap()
                .create_vcpu(vcpu_id, vm_exit_handler.clone())?;
        }

        let vm = Vm {
            _vm_instance: vm_instance,
            vcpu_manager,
            memory_address_space,
            irq_chip,
            device_manager,
            gdb_stub: vm_config.gdb_port.map(GdbStub::new),
            monitor: monitor_server_builder.build(),
            vm_config,
        };

        self.vm = Some(vm);

        Ok(())
    }

    pub fn run(&mut self, boot_loader: &dyn BootLoader) -> Result<()> {
        self.vm
            .as_mut()
            .ok_or(Error::VmNotExists)?
            .boot(boot_loader)?;

        loop {}
    }
}
