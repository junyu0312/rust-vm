use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::anyhow;
use vm_bootloader::boot_loader::BootLoader;
use vm_core::device::device_manager::DeviceManager;
use vm_core::device::mmio::MmioLayout;
use vm_core::layout::MemoryLayout;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::mm::region::MemoryRegion;
use vm_core::virt::Virt;

use crate::device::init_device;
use crate::vm::error::Error;

pub mod error;

pub type Result<T> = core::result::Result<T, Error>;

pub struct VmBuilder<V> {
    memory_size: usize,
    vcpus: usize,
    _mark: PhantomData<V>,
}

impl<V> VmBuilder<V> {
    pub fn new(memory_size: usize, vcpus: usize) -> Self {
        VmBuilder {
            memory_size,
            vcpus,
            _mark: PhantomData,
        }
    }
}

pub struct Vm<V: Virt> {
    memory: Arc<Mutex<MemoryAddressSpace<V::Memory>>>,
    virt: V,
    irq_chip: Arc<V::Irq>,
    device_manager: DeviceManager,
}

impl<V> VmBuilder<V>
where
    V: Virt,
{
    fn init_mm<C>(&self, ram_base: u64) -> Result<MemoryAddressSpace<C>>
    where
        C: MemoryContainer,
    {
        let memory_region = MemoryRegion::placeholder(ram_base, self.memory_size);

        let mut memory_regions = MemoryAddressSpace::default();
        memory_regions
            .try_insert(memory_region)
            .map_err(|_| anyhow!("Failed to insert memory_region"))
            .map_err(|err| Error::InitMemory(err.to_string()))?;

        Ok(memory_regions)
    }

    pub fn build(&self) -> Result<Vm<V>> {
        let mut virt = V::new()?;

        let irq_chip = virt
            .init_irq()
            .map_err(|err| Error::InitIrqchip(err.to_string()))?;

        virt.init_vcpus(self.vcpus)
            .map_err(|err| Error::InitCpu(err.to_string()))?;

        let layout = virt.get_layout();
        let mmio_layout = MmioLayout::new(layout.get_mmio_start(), layout.get_mmio_len());

        let mut memory = self.init_mm(layout.get_ram_base())?;
        virt.init_memory(&mmio_layout, &mut memory, self.memory_size as u64)
            .map_err(|err| Error::InitMemory(err.to_string()))?;
        let memory = Arc::new(Mutex::new(memory));

        virt.post_init()
            .map_err(|err| Error::PostInit(err.to_string()))?;

        let mut device_manager = DeviceManager::new(mmio_layout);
        init_device(memory.clone(), &mut device_manager, irq_chip.clone())
            .map_err(|err| Error::InitDevice(err.to_string()))?;

        let vm = Vm {
            memory,
            virt,
            irq_chip,
            device_manager,
        };

        Ok(vm)
    }
}

impl<V> Vm<V>
where
    V: Virt,
{
    pub fn load(&mut self, boot_loader: &dyn BootLoader<V>) -> Result<()> {
        let mut memory = self.memory.lock().unwrap();

        boot_loader.load(
            &mut self.virt,
            &mut memory,
            &self.irq_chip,
            self.device_manager.mmio_devices(),
        )?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        self.virt
            .run(&mut self.device_manager)
            .map_err(|err| Error::Run(err.to_string()))?;

        Ok(())
    }
}
