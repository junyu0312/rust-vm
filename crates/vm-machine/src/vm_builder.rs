use std::sync::Arc;

use vm_core::arch::layout::MemoryLayout;
use vm_core::debug::gdbstub::GdbStub;
use vm_core::device::device_manager::DeviceManager;
use vm_core::device::mmio::MmioLayout;
use vm_core::monitor::MonitorServerBuilder;
use vm_core::virt::Virt;
use vm_device::device::Device;
use vm_mm::manager::MemoryAddressSpace;

use crate::device::InitDevice;
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
        let mut monitor_server_builder = MonitorServerBuilder::default();

        let mut virt = V::new(self.vcpus)?;

        let mut memory_regions = MemoryAddressSpace::default();
        virt.init_memory(&mut memory_regions, self.memory_size)?;
        let memory = Arc::new(memory_regions);

        let layout = virt.get_layout();
        let mmio_layout = MmioLayout::new(layout.get_mmio_start(), layout.get_mmio_len());

        let irq_chip = if !self.devices.iter().any(Device::is_irq_chip) {
            virt.init_irq()?
        } else {
            todo!()
        };

        let mut device_manager = DeviceManager::new(mmio_layout);
        device_manager.init_devices(
            &mut monitor_server_builder,
            memory.clone(),
            self.devices,
            irq_chip.clone(),
        )?;

        let vm = Vm {
            memory,
            virt,
            irq_chip,
            device_manager: Arc::new(device_manager),
            gdb_stub: self.gdb_port.map(GdbStub::new),
            monitor: monitor_server_builder.build(),
        };

        Ok(vm)
    }
}
