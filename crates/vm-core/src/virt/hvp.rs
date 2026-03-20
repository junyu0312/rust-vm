use std::cell::OnceCell;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

use applevisor::gic::GicConfig;
use applevisor::memory::MemPerms;
use applevisor::vm::GicEnabled;
use applevisor::vm::VirtualMachine;
use applevisor::vm::VirtualMachineConfig;
use applevisor::vm::VirtualMachineInstance;
use vm_mm::allocator::Allocator;
use vm_mm::manager::MemoryAddressSpace;
use vm_mm::region::MemoryRegion;

use crate::arch::Arch;
use crate::arch::aarch64::AArch64;
use crate::arch::aarch64::firmware::psci::psci_0_2::Psci02;
use crate::arch::aarch64::layout::AArch64Layout;
use crate::arch::aarch64::vcpu::AArch64Vcpu;
use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::arch::aarch64::vcpu::reg::cnthctl_el2::CnthctlEl2;
use crate::arch::aarch64::vcpu::reg::sctlr_el1::SctlrEl1;
use crate::arch::aarch64::vm_exit::HandleVmExitResult;
use crate::arch::aarch64::vm_exit::handle_vm_exit;
use crate::arch::irq::InterruptController;
use crate::arch::layout::MemoryLayout;
use crate::arch::vcpu::Vcpu;
use crate::device::vm_exit::DeviceVmExitHandler;
use crate::error::Error;
use crate::error::Result;
use crate::virt::Virt;
use crate::virt::hvp::irq_chip::HvpGicV3;
use crate::virt::hvp::mm::HvpAllocator;
use crate::virt::hvp::mm::MemoryWrapper;
use crate::virt::hvp::vcpu::HvpVcpu;

pub(crate) mod vcpu;

mod irq_chip;
mod mm;

fn setup_cpu<C>(dtb_start: u64, start_pc: u64, cpu_id: usize, vcpu: &mut C) -> anyhow::Result<()>
where
    C: AArch64Vcpu,
{
    vcpu.set_sys_reg(SysRegister::MpidrEl1, cpu_id as u64)?;

    if cpu_id == 0 {
        // Setup general-purpose register
        vcpu.set_core_reg(CoreRegister::X0, dtb_start)?;
        vcpu.set_core_reg(CoreRegister::X1, 0)?;
        vcpu.set_core_reg(CoreRegister::X2, 0)?;
        vcpu.set_core_reg(CoreRegister::X3, 0)?;
        vcpu.set_core_reg(CoreRegister::PC, start_pc)?;
    } else {
        vcpu.set_core_reg(CoreRegister::X0, 0)?;
        vcpu.set_core_reg(CoreRegister::X1, 0)?;
        vcpu.set_core_reg(CoreRegister::X2, 0)?;
        vcpu.set_core_reg(CoreRegister::X3, 0)?;
        vcpu.set_core_reg(CoreRegister::PC, 0)?;
    }

    {
        // CPU mode

        let mut pstate = vcpu.get_core_reg(CoreRegister::PState)?;
        pstate |= 0x03C0; // DAIF
        pstate &= !0xf; // Clear low 4 bits
        pstate |= 0x0005; // El1h
        vcpu.set_core_reg(CoreRegister::PState, pstate)?;

        // more, non secure el1
        if false {
            todo!()
        }
    }

    {
        // Caches, MMUs

        let mut sctlr_el1 = vcpu.get_sctlr_el1()?;
        sctlr_el1.remove(SctlrEl1::M); // Disable MMU
        sctlr_el1.remove(SctlrEl1::I); // Disable I-cache
        vcpu.set_sctlr_el1(sctlr_el1)?;
    }

    {
        // Architected timers

        if false {
            todo!(
                "CNTFRQ must be programmed with the timer frequency and CNTVOFF must be programmed with a consistent value on all CPUs."
            );
        }

        if false {
            // MacOS get panic, should we enable this in Linux?
            let mut cnthctl_el2 = vcpu.get_cnthctl_el2()?;
            cnthctl_el2.insert(CnthctlEl2::EL1PCTEN); // TODO: or bit0?(https://www.kernel.org/doc/html/v5.3/arm64/booting.html)
            vcpu.set_cnthctl_el2(cnthctl_el2)?;
        }
    }

    {
        // Coherency

        // Do nothing
    }

    {
        // System registers

        if false {
            todo!()
        }
    }

    anyhow::Ok(())
}

pub struct Hvp {
    arch: AArch64,
    vm: VirtualMachineInstance<GicEnabled>,
    gic_chip: Arc<HvpGicV3>,
    vcpus: OnceCell<Vec<HvpVcpu>>,
    num_vcpus: usize,
    psci: Arc<Psci02>,
    cpu_on_receiver: Option<Vec<Receiver<(u64, u64)>>>,
}

impl Virt for Hvp {
    type Arch = AArch64;
    type Vcpu = HvpVcpu;
    type Memory = MemoryWrapper;

    fn new(num_vcpus: usize) -> Result<Self> {
        let layout = AArch64Layout::default();

        let mut vm_config = VirtualMachineConfig::default();
        vm_config
            .set_el2_enabled(true)
            .map_err(|err| Error::FailedInitialize(err.to_string()))?;

        let mut gic_config = GicConfig::default();

        let distributor_base = layout.get_distributor_start();
        let redistributor_base = layout.get_redistributor_start();
        let msi_base = layout.get_msi_start();
        let distributor_base_alignment = GicConfig::get_distributor_base_alignment()
            .map_err(|err| Error::InterruptControllerFailed(err.to_string()))?;
        let redistributor_base_alignment = GicConfig::get_redistributor_base_alignment()
            .map_err(|err| Error::InterruptControllerFailed(err.to_string()))?;
        let msi_region_base_alignment = GicConfig::get_msi_region_base_alignment()
            .map_err(|err| Error::InterruptControllerFailed(err.to_string()))?;

        let gic_distributor_size = GicConfig::get_distributor_size().map_err(|err| {
            Error::InterruptControllerFailed(format!(
                "Failed to get the size of distributor, {:?}",
                err
            ))
        })?;
        layout.set_distributor_len(gic_distributor_size).unwrap();

        let gic_redistributor_region_size =
            GicConfig::get_redistributor_region_size().map_err(|err| {
                Error::InterruptControllerFailed(format!(
                    "Failed to get the size of redistributor region, {:?}",
                    err
                ))
            })?;
        layout
            .set_redistributor_region_len(gic_redistributor_region_size)
            .unwrap();

        let gic_msi_region_size = GicConfig::get_msi_region_size().map_err(|err| {
            Error::InterruptControllerFailed(format!(
                "Failed to get the size of msi region, {:?}",
                err
            ))
        })?;
        layout.set_msi_region_len(gic_msi_region_size).unwrap();

        {
            // Setup distributor
            if !distributor_base.is_multiple_of(distributor_base_alignment as u64) {
                return Err(Error::InterruptControllerFailed(
                    "The base address of gic distributor is not aligned".to_string(),
                ));
            }
            gic_config
                .set_distributor_base(distributor_base)
                .map_err(|err| Error::FailedInitialize(err.to_string()))?;
        }

        {
            // Setup redistributor
            if !redistributor_base.is_multiple_of(redistributor_base_alignment as u64) {
                return Err(Error::InterruptControllerFailed(
                    "The base address of gic redistributor is not aligned".to_string(),
                ));
            }
            if distributor_base + gic_distributor_size as u64 > redistributor_base {
                return Err(Error::InterruptControllerFailed(
                    "distributor too large".to_string(),
                ));
            }
            gic_config
                .set_redistributor_base(redistributor_base)
                .map_err(|err| Error::FailedInitialize(err.to_string()))?;
        }

        {
            // Setup msi
            if !msi_base.is_multiple_of(msi_region_base_alignment as u64) {
                return Err(Error::InterruptControllerFailed(
                    "The base address of gic msi is not aligned".to_string(),
                ));
            }
            if redistributor_base + gic_redistributor_region_size as u64 > msi_base {
                return Err(Error::InterruptControllerFailed(
                    "redistributor too large".to_string(),
                ));
            }
            gic_config
                .set_msi_region_base(msi_base)
                .map_err(|err| Error::FailedInitialize(err.to_string()))?;
            gic_config
                .set_msi_interrupt_range(256, 256)
                .map_err(|err| Error::FailedInitialize(err.to_string()))?;
        }

        if msi_base + gic_msi_region_size as u64 > layout.get_ram_base() {
            return Err(Error::InterruptControllerFailed(
                "msi region too large".to_string(),
            ));
        }

        let vm = VirtualMachine::with_gic(vm_config, gic_config).map_err(|err| {
            Error::FailedInitialize(
                format!("hvp: Failed to create a vm instance, reason: {}", err).to_string(),
            )
        })?;

        let gic_chip = HvpGicV3::new(distributor_base, redistributor_base, msi_base, vm.clone());
        let gic_chip = Arc::new(gic_chip);

        let mut cpu_on_receiver = vec![];
        let mut cpu_on_barrier = vec![];
        for _ in 0..num_vcpus {
            let (tx, rx) = mpsc::channel();
            cpu_on_receiver.push(rx);
            cpu_on_barrier.push(tx);
        }

        Ok(Hvp {
            arch: AArch64 { layout },
            vm,
            gic_chip,
            vcpus: OnceCell::default(),
            num_vcpus,
            psci: Arc::new(Psci02 { cpu_on_barrier }),
            cpu_on_receiver: Some(cpu_on_receiver),
        })
    }

    fn init_irq(&mut self) -> Result<Arc<dyn InterruptController>> {
        Ok(self.gic_chip.clone())
    }

    fn init_memory(
        &mut self,
        memory_address_space: &mut MemoryAddressSpace<MemoryWrapper>,
        memory_size: usize,
    ) -> Result<()> {
        let allocator = HvpAllocator { vm: &self.vm };

        let ram_base = self.get_layout().get_ram_base();
        let mut memory = allocator.alloc(memory_size, None)?;
        memory.0.map(ram_base, MemPerms::ReadWriteExec)?;
        memory_address_space
            .try_insert(MemoryRegion::new(ram_base, memory))
            .map_err(|_| Error::FailedInitialize("Failed to initialize memory".to_string()))?;

        self.get_layout_mut().set_ram_size(memory_size as u64)?;

        Ok(())
    }

    fn get_layout(&self) -> &AArch64Layout {
        self.arch.get_layout()
    }

    fn get_layout_mut(&mut self) -> &mut AArch64Layout {
        self.arch.get_layout_mut()
    }

    fn get_vcpu_number(&self) -> usize {
        self.num_vcpus
    }

    fn get_vcpu_mut(&mut self, vcpu_id: u64) -> Result<Option<&mut HvpVcpu>> {
        Ok(self
            .vcpus
            .get_mut()
            .ok_or_else(|| Error::Internal("vcpu is not initialized".to_string()))?
            .get_mut(vcpu_id as usize))
    }

    fn get_vcpus(&self) -> Result<&Vec<Self::Vcpu>> {
        self.vcpus
            .get()
            .ok_or_else(|| Error::Internal("vcpu is not initialized".to_string()))
    }

    fn get_vcpus_mut(&mut self) -> Result<&mut Vec<Self::Vcpu>> {
        self.vcpus
            .get_mut()
            .ok_or_else(|| Error::Internal("vcpu is not initialized".to_string()))
    }

    fn run(&mut self, device_manager: Arc<dyn DeviceVmExitHandler>) -> Result<()> {
        let mmio_layout = device_manager.mmio_layout();

        thread::scope(|s| {
            let cpu_on_receiver = self.cpu_on_receiver.take().unwrap();

            for (vcpu_id, rx) in (0..self.num_vcpus).zip(cpu_on_receiver.into_iter()) {
                let mmio_layout = mmio_layout.clone();
                let device_manager = device_manager.clone();
                let vm = self.vm.clone();
                let psci = self.psci.clone();

                let layout = self.get_layout().clone();

                s.spawn(move || -> anyhow::Result<()> {
                    let vcpu = vm.vcpu_create()?;

                    let mut vcpu = HvpVcpu::new(vcpu_id as u64, vcpu);

                    setup_cpu(
                        layout.get_dtb_start(),
                        layout.get_start_pc().unwrap(),
                        vcpu_id,
                        &mut vcpu,
                    )?;

                    if vcpu_id != 0 {
                        let (pc, context_id) = rx.recv().unwrap();
                        vcpu.set_core_reg(CoreRegister::PC, pc)?;
                        vcpu.set_core_reg(CoreRegister::X0, context_id)?;
                    }

                    loop {
                        let vm_exit_info = vcpu.run(&mmio_layout)?;

                        match handle_vm_exit(
                            &vcpu,
                            vm_exit_info,
                            psci.as_ref(),
                            device_manager.as_ref(),
                        )? {
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
