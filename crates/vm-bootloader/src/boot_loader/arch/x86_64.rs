use std::path::Path;
use std::path::PathBuf;
use std::slice::Iter;

use async_trait::async_trait;
use vm_core::arch::irq::InterruptController;
use vm_core::arch::x86_64::layout::ACPI_MAX_LEN;
use vm_core::arch::x86_64::layout::ACPI_RSDT_START;
use vm_core::arch::x86_64::layout::APIC_ADDR;
use vm_core::arch::x86_64::layout::BOOT_PARAMS_START;
use vm_core::arch::x86_64::layout::CMDLINE_START;
use vm_core::arch::x86_64::layout::ECAM_BASE;
use vm_core::arch::x86_64::layout::ECAM_LENGTH;
use vm_core::arch::x86_64::layout::GDT_START;
use vm_core::arch::x86_64::layout::INITRD_START;
use vm_core::arch::x86_64::layout::IOAPIC_ADDR;
use vm_core::arch::x86_64::layout::KERNEL_START;
use vm_core::arch::x86_64::layout::MMIO_LEN;
use vm_core::arch::x86_64::layout::MMIO_START;
use vm_core::cpu::vcpu::Vcpu;
use vm_core::device::Device;
use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;

use crate::boot_loader::BootLoader;
use crate::boot_loader::BootLoaderBuilder;
use crate::boot_loader::Result;
use crate::initrd_loader::InitrdLoadResult;
use crate::initrd_loader::InitrdLoader;
use crate::kernel_loader::linux::bzimage::BzImage;
use crate::kernel_loader::linux::bzimage::BzImageBootParams;
use crate::kernel_loader::linux::bzimage::LoadResult;
use crate::utils::aml::build_definition_block;

pub struct X86_64BootLoader {
    kernel: PathBuf,
    initramfs: Option<PathBuf>,
    cmdline: Option<String>,
}

impl X86_64BootLoader {
    fn load_initramfs(
        &self,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        initramfs: &Path,
    ) -> Result<InitrdLoadResult> {
        let result =
            InitrdLoader::new(initramfs)?.load(ram_allocator, memory, INITRD_START as u64)?;

        Ok(result)
    }

    fn load_image(
        &self,
        vcpus: usize,
        definition_block: Vec<u8>,
        initrd_load_result: Option<InitrdLoadResult>,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
    ) -> Result<LoadResult> {
        let params = BzImageBootParams {
            vcpus,
            definition_block,
            gdt_start: GDT_START,
            boot_params_start: BOOT_PARAMS_START,
            cmdline_start: CMDLINE_START,
            acpi_rsdt_addr: ACPI_RSDT_START,
            acpi_max_length: ACPI_MAX_LEN,
            kernel_start: KERNEL_START,
            initrd: initrd_load_result,
            mmio_start: MMIO_START,
            mmio_length: MMIO_LEN,
            ecam_base: ECAM_BASE,
            ecam_length: ECAM_LENGTH,
            ioapic_base_addr: IOAPIC_ADDR,
            apic_base_addr: APIC_ADDR,
        };

        let load_result = BzImage::new(&self.kernel, self.cmdline.as_deref())?.load(
            ram_allocator,
            memory,
            &params,
        )?;

        Ok(load_result)
    }
}

impl BootLoaderBuilder for X86_64BootLoader {
    fn new(kernel: PathBuf, initramfs: Option<PathBuf>, cmdline: Option<String>) -> Self {
        X86_64BootLoader {
            kernel,
            initramfs,
            cmdline,
        }
    }
}

#[async_trait]
impl BootLoader for X86_64BootLoader {
    async fn load(
        &self,
        _ram_size: u64,
        vcpus: usize,
        boot_vcpu: &mut Vcpu,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        _irq_chip: &dyn InterruptController,
        devices: Iter<'_, Box<dyn Device>>,
    ) -> Result<()> {
        let definition_block = build_definition_block(devices);

        let initrd_load_result = if let Some(initramfs) = &self.initramfs {
            Some(self.load_initramfs(ram_allocator, memory, initramfs)?)
        } else {
            None
        };

        let load_result = self.load_image(
            vcpus,
            definition_block,
            initrd_load_result,
            ram_allocator,
            memory,
        )?;

        boot_vcpu
            .setup_vcpu(
                load_result.gdt,
                load_result.start_pc,
                load_result.gdt_start,
                load_result.boot_params_start,
            )
            .await?;

        Ok(())
    }
}
