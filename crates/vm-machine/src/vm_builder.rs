use std::sync::Arc;
use std::sync::Mutex;

use vm_core::arch::layout::MemoryLayout;
use vm_core::debug::gdbstub::GdbStub;
use vm_core::device::device_manager::DeviceManager;
use vm_core::device::mmio::MmioLayout;
use vm_core::virt::Virt;
use vm_device::device::Device;
use vm_mm::allocator::MemoryContainer;
use vm_mm::manager::MemoryAddressSpace;
use vm_mm::region::MemoryRegion;

use crate::device::InitDevice;
use crate::error::Error;
use crate::error::Result;
use crate::vm::Vm;

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
        let mut virt = V::new(self.vcpus)?;

        let layout = virt.get_layout().clone();
        let mmio_layout = MmioLayout::new(layout.get_mmio_start(), layout.get_mmio_len());

        let irq_chip = if !self.devices.iter().any(Device::is_irq_chip) {
            Some(virt.init_irq()?)
        } else {
            None
        };

        let mut memory = self.init_mm(layout.get_ram_base())?;
        virt.init_memory(&mmio_layout, &mut memory, self.memory_size as u64)?;
        let memory = Arc::new(Mutex::new(memory));

        virt.post_init()?;

        let mut device_manager = DeviceManager::new(mmio_layout);
        device_manager
            .init_devices(memory.clone(), self.devices, irq_chip)
            .map_err(|err| Error::InitDevice(err.to_string()))?;

        let vm = Vm {
            memory,
            virt,
            device_manager: Arc::new(Mutex::new(device_manager)),
            gdb_stub: self.gdb_port.map(GdbStub::new),
        };

        Ok(vm)
    }

    fn init_mm<C>(&self, ram_base: u64) -> Result<MemoryAddressSpace<C>>
    where
        C: MemoryContainer,
    {
        let memory_region = MemoryRegion::placeholder(ram_base, self.memory_size);

        let mut memory_regions = MemoryAddressSpace::default();
        memory_regions
            .try_insert(memory_region)
            .map_err(|_| Error::InitMemory("Failed to insert memory_region".to_string()))?;

        Ok(memory_regions)
    }
}
