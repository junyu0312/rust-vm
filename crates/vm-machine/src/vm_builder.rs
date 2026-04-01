use std::sync::Arc;

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
use vm_core::debug::gdbstub::GdbStub;
use vm_core::device::mmio::layout::MmioLayout;
use vm_core::device_manager::manager::DeviceManager;
use vm_core::error::Error;
use vm_core::monitor::MonitorServerBuilder;
use vm_core::virt::SetUserMemoryRegionFlags;
use vm_core::virt::Virt;
use vm_device::device::Device;
use vm_mm::allocator::Allocator;
use vm_mm::allocator::std_allocator::StdAllocator;
use vm_mm::manager::MemoryAddressSpace;
use vm_mm::memory_container::MemoryContainer;
use vm_mm::region::MemoryRegion;

use crate::device::InitDevice;
use crate::error::Result;
use crate::vm::Vm;

const PAGE_SIZE: usize = 4 << 10;

pub struct VmBuilder {
    memory_size: usize,
    vcpus: usize,
    devices: Vec<Device>,
    gdb_port: Option<u16>,
}

impl VmBuilder {
    pub fn new(
        memory_size: usize,
        vcpus: usize,
        devices: Vec<Device>,
        gdb_port: Option<u16>,
    ) -> Self {
        VmBuilder {
            memory_size,
            vcpus,
            devices,
            gdb_port,
        }
    }

    pub fn build<V>(self) -> Result<Vm<V>>
    where
        V: Virt,
    {
        let mut monitor_server_builder = MonitorServerBuilder::default();

        let mut virt = V::new(self.vcpus)?;

        let mut memory_address_space = MemoryAddressSpace::default();
        {
            let memory_region = StdAllocator.alloc(self.memory_size, Some(PAGE_SIZE))?;

            virt.set_user_memory_region(
                memory_region.hva() as _,
                RAM_BASE,
                self.memory_size,
                SetUserMemoryRegionFlags::ReadWriteExec,
            )?;
            memory_address_space
                .try_insert(MemoryRegion::new(RAM_BASE, Box::new(memory_region)))
                .map_err(|_| Error::FailedInitialize("Failed to initialize memory".to_string()))?;
        }

        let memory_address_space = Arc::new(memory_address_space);

        let mmio_layout = MmioLayout::new(MMIO_START, MMIO_LEN);

        let irq_chip = if !self.devices.iter().any(Device::is_irq_chip) {
            virt.create_irq_chip()?
        } else {
            todo!()
        };

        let mut device_manager = DeviceManager::new(mmio_layout);
        device_manager.init_devices(
            &mut monitor_server_builder,
            memory_address_space.clone(),
            self.devices,
            irq_chip.clone(),
        )?;

        let vm = Vm {
            ram_size: self.memory_size as u64,
            vcpus: self.vcpus,
            memory_address_space,
            virt,
            irq_chip,
            device_manager,
            gdb_stub: self.gdb_port.map(GdbStub::new),
            monitor: monitor_server_builder.build(),
        };

        Ok(vm)
    }
}
