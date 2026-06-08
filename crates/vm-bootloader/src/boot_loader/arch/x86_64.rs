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
use vm_core::arch::x86_64::layout::PCI_BAR_MMIO_WINDOW_LENGTH;
use vm_core::arch::x86_64::layout::PCI_BAR_MMIO_WINDOW_START;
use vm_core::cpu::vcpu::Vcpu;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;

use crate::boot_loader::BootLoader;
use crate::boot_loader::BootLoaderBuilder;
use crate::boot_loader::Result;
use crate::kernel_loader::KernelLoader;
use crate::kernel_loader::linux::bzimage::BzImage;
use crate::kernel_loader::linux::bzimage::BzImageBootParams;

pub struct X86_64BootLoader {
    kernel: PathBuf,
    initramfs: Option<PathBuf>,
    cmdline: Option<String>,
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
        ram_size: u64,
        vcpus: usize,
        boot_vcpu: &mut Vcpu,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        _irq_chip: &dyn InterruptController,
        _devices: Iter<'_, Box<dyn MmioDevice>>,
    ) -> Result<()> {
        let mut kernel_loader = BzImage::new(
            &self.kernel,
            self.initramfs.as_deref(),
            self.cmdline.as_deref(),
        )?;

        let params = BzImageBootParams {
            vcpus,
            memory_size: ram_size,
            definition_block: vec![], // TODO
            gdt_start: GDT_START,
            boot_params_start: BOOT_PARAMS_START,
            cmdline_start: CMDLINE_START,
            acpi_rsdt_addr: ACPI_RSDT_START,
            acpi_max_length: ACPI_MAX_LEN,
            kernel_start: KERNEL_START,
            initrd_start: INITRD_START,
            mmio_start: MMIO_START,
            mmio_length: MMIO_LEN,
            pci_bar_mmio_window_start: PCI_BAR_MMIO_WINDOW_START,
            pci_bar_mmio_window_length: PCI_BAR_MMIO_WINDOW_LENGTH,
            ecam_base: ECAM_BASE,
            ecam_length: ECAM_LENGTH,
            ioapic_base_addr: IOAPIC_ADDR,
            apic_base_addr: APIC_ADDR,
        };

        let load_result = kernel_loader.load(ram_allocator, memory, &params)?;

        boot_vcpu
            .setup_vcpu(
                load_result.gdt,
                load_result.kernel_start as u32,
                params.gdt_start,
                params.boot_params_start,
            )
            .await?;

        Ok(())
    }
}
