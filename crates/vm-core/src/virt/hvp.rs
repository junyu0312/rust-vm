use std::cell::OnceCell;
use std::sync::Arc;
use std::sync::Barrier;
use std::sync::Mutex;
use std::thread;

use anyhow::anyhow;
use applevisor::gic::GicConfig;
use applevisor::memory::MemPerms;
use applevisor::vm::GicEnabled;
use applevisor::vm::VirtualMachine;
use applevisor::vm::VirtualMachineConfig;
use applevisor::vm::VirtualMachineInstance;
use applevisor_sys::hv_sys_reg_t;

use crate::arch::Arch;
use crate::arch::aarch64::AArch64;
use crate::arch::vm_exit::aarch64::HandleVmExitResult;
use crate::arch::vm_exit::aarch64::handle_vm_exit;
use crate::device::mmio::MmioLayout;
use crate::device::vm_exit::DeviceVmExitHandler;
use crate::irq::InterruptController;
use crate::layout::MemoryLayout;
use crate::layout::aarch64::AArch64Layout;
use crate::mm::manager::MemoryAddressSpace;
use crate::vcpu::Vcpu;
use crate::vcpu::arch::aarch64::AArch64Vcpu;
use crate::vcpu::arch::aarch64::reg::CoreRegister;
use crate::virt::Virt;
use crate::virt::VirtError;
use crate::virt::hvp::irq_chip::HvpGicV3;
use crate::virt::hvp::mm::HvpAllocator;
use crate::virt::hvp::mm::MemoryWrapper;
use crate::virt::hvp::vcpu::HvpVcpu;

pub(crate) mod vcpu;

mod irq_chip;
mod mm;

pub struct Hvp {
    arch: AArch64,
    vm: VirtualMachineInstance<GicEnabled>,
    gic_chip: Arc<HvpGicV3>,
    vcpus: OnceCell<Vec<HvpVcpu>>,
    num_vcpus: usize,
}

impl Virt for Hvp {
    type Arch = AArch64;
    type Vcpu = HvpVcpu;
    type Memory = MemoryWrapper;

    fn new(num_vcpus: usize) -> Result<Self, VirtError> {
        let layout = AArch64Layout::default();

        let mut vm_config = VirtualMachineConfig::default();
        vm_config
            .set_el2_enabled(true)
            .map_err(|err| VirtError::FailedInitialize(err.to_string()))?;

        let mut gic_config = GicConfig::default();

        let distributor_base = layout.get_distributor_start();
        let redistributor_base = layout.get_redistributor_start();
        let msi_base = layout.get_msi_start();
        let distributor_base_alignment = GicConfig::get_distributor_base_alignment()
            .map_err(|err| VirtError::InterruptControllerFailed(err.to_string()))?;
        let redistributor_base_alignment = GicConfig::get_redistributor_base_alignment()
            .map_err(|err| VirtError::InterruptControllerFailed(err.to_string()))?;
        let msi_region_base_alignment = GicConfig::get_msi_region_base_alignment()
            .map_err(|err| VirtError::InterruptControllerFailed(err.to_string()))?;

        let gic_distributor_size = GicConfig::get_distributor_size().map_err(|err| {
            VirtError::InterruptControllerFailed(format!(
                "Failed to get the size of distributor, {:?}",
                err
            ))
        })?;
        layout.set_distributor_len(gic_distributor_size).unwrap();

        let gic_redistributor_region_size =
            GicConfig::get_redistributor_region_size().map_err(|err| {
                VirtError::InterruptControllerFailed(format!(
                    "Failed to get the size of redistributor region, {:?}",
                    err
                ))
            })?;
        layout
            .set_redistributor_region_len(gic_redistributor_region_size)
            .unwrap();

        let gic_msi_region_size = GicConfig::get_msi_region_size().map_err(|err| {
            VirtError::InterruptControllerFailed(format!(
                "Failed to get the size of msi region, {:?}",
                err
            ))
        })?;
        layout.set_msi_region_len(gic_msi_region_size).unwrap();

        {
            // Setup distributor
            if !distributor_base.is_multiple_of(distributor_base_alignment as u64) {
                return Err(VirtError::InterruptControllerFailed(
                    "The base address of gic distributor is not aligned".to_string(),
                ));
            }
            gic_config
                .set_distributor_base(distributor_base)
                .map_err(|err| VirtError::FailedInitialize(err.to_string()))?;
        }

        {
            // Setup redistributor
            if !redistributor_base.is_multiple_of(redistributor_base_alignment as u64) {
                return Err(VirtError::InterruptControllerFailed(
                    "The base address of gic redistributor is not aligned".to_string(),
                ));
            }
            if distributor_base + gic_distributor_size as u64 > redistributor_base {
                return Err(VirtError::InterruptControllerFailed(
                    "distributor too large".to_string(),
                ));
            }
            gic_config
                .set_redistributor_base(redistributor_base)
                .map_err(|err| VirtError::FailedInitialize(err.to_string()))?;
        }

        {
            // Setup msi
            if !msi_base.is_multiple_of(msi_region_base_alignment as u64) {
                return Err(VirtError::InterruptControllerFailed(
                    "The base address of gic msi is not aligned".to_string(),
                ));
            }
            if redistributor_base + gic_redistributor_region_size as u64 > msi_base {
                return Err(VirtError::InterruptControllerFailed(
                    "redistributor too large".to_string(),
                ));
            }
            gic_config
                .set_msi_region_base(msi_base)
                .map_err(|err| VirtError::FailedInitialize(err.to_string()))?;
            gic_config
                .set_msi_interrupt_range(256, 256)
                .map_err(|err| VirtError::FailedInitialize(err.to_string()))?;
        }

        if msi_base + gic_msi_region_size as u64 > layout.get_ram_base() {
            return Err(VirtError::InterruptControllerFailed(
                "msi region too large".to_string(),
            ));
        }

        let vm = VirtualMachine::with_gic(vm_config, gic_config).map_err(|err| {
            VirtError::FailedInitialize(
                format!("hvp: Failed to create a vm instance, reason: {}", err).to_string(),
            )
        })?;

        let gic_chip = HvpGicV3::new(distributor_base, redistributor_base, msi_base, vm.clone());
        let gic_chip = Arc::new(gic_chip);

        Ok(Hvp {
            arch: AArch64 { layout },
            vm,
            gic_chip,
            vcpus: OnceCell::default(),
            num_vcpus,
        })
    }

    fn init_irq(&mut self) -> anyhow::Result<Arc<dyn InterruptController>> {
        Ok(self.gic_chip.clone())
    }

    fn init_memory(
        &mut self,
        _mmio_layout: &MmioLayout,
        memory: &mut MemoryAddressSpace<MemoryWrapper>,
        memory_size: u64,
    ) -> anyhow::Result<()> {
        let allocator = HvpAllocator { vm: &self.vm };

        for region in memory {
            region.alloc(&allocator)?;

            let memory = region.memory.get_mut().unwrap();
            memory.0.map(region.gpa, MemPerms::ReadWriteExec)?;
        }

        self.get_layout_mut().set_ram_size(memory_size)?;

        Ok(())
    }

    fn post_init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_layout(&self) -> &AArch64Layout {
        self.arch.get_layout()
    }

    fn get_layout_mut(&mut self) -> &mut AArch64Layout {
        self.arch.get_layout_mut()
    }

    fn get_vcpu_mut(&mut self, vcpu_id: u64) -> anyhow::Result<Option<&mut HvpVcpu>> {
        let vcpu = self
            .vcpus
            .get_mut()
            .ok_or_else(|| anyhow!("vcpu is not initialized"))?
            .get_mut(vcpu_id as usize);

        Ok(vcpu)
    }

    fn get_vcpus(&self) -> anyhow::Result<&Vec<Self::Vcpu>> {
        let vcpus = self
            .vcpus
            .get()
            .ok_or_else(|| anyhow!("vcpu is not initialized"))?;

        Ok(vcpus)
    }

    fn get_vcpus_mut(&mut self) -> anyhow::Result<&mut Vec<Self::Vcpu>> {
        let vcpus = self
            .vcpus
            .get_mut()
            .ok_or_else(|| anyhow!("vcpu is not initialized"))?;

        Ok(vcpus)
    }

    fn run(&mut self, device_manager: Arc<Mutex<dyn DeviceVmExitHandler>>) -> anyhow::Result<()> {
        let mmio_layout = device_manager.lock().unwrap().mmio_layout();

        let barrier = Barrier::new(self.num_vcpus);

        thread::scope(|s| {
            for vcpu_id in 0..self.num_vcpus {
                let mmio_layout = mmio_layout.clone();
                let device_manager = device_manager.clone();
                let vm = self.vm.clone();
                let barrier = &barrier;

                s.spawn(move || -> anyhow::Result<()> {
                    let vcpu = vm.vcpu_create()?;
                    vcpu.set_sys_reg(hv_sys_reg_t::MPIDR_EL1, vcpu_id as u64)?;

                    let mut vcpu = HvpVcpu::new(vcpu_id as u64, vcpu);

                    barrier.wait();

                    loop {
                        let vm_exit_info = vcpu.run(&mmio_layout)?;

                        match handle_vm_exit(&vcpu, vm_exit_info, device_manager.clone())? {
                            HandleVmExitResult::Continue => (),
                            HandleVmExitResult::NextInstruction => {
                                let pc = vcpu.get_core_reg(CoreRegister::PC)?;
                                vcpu.set_core_reg(CoreRegister::PC, pc + 4)?;
                            }
                        }
                    }
                });
            }
        });

        Ok(())
    }
}
