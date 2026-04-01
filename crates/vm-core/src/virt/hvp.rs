use std::ptr::null_mut;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

use applevisor::gic::GicConfig;
use applevisor::memory::MemPerms;
use applevisor::prelude::HypervisorError;
use applevisor_sys::hv_error_t;
use applevisor_sys::hv_gic_config_create;
use applevisor_sys::hv_gic_config_set_distributor_base;
use applevisor_sys::hv_gic_config_set_msi_interrupt_range;
use applevisor_sys::hv_gic_config_set_msi_region_base;
use applevisor_sys::hv_gic_config_set_redistributor_base;
use applevisor_sys::hv_gic_config_t;
use applevisor_sys::hv_gic_create;
use applevisor_sys::hv_vcpu_create;
use applevisor_sys::hv_vcpu_exit_t;
use applevisor_sys::hv_vm_config_create;
use applevisor_sys::hv_vm_config_set_el2_enabled;
use applevisor_sys::hv_vm_create;
use applevisor_sys::hv_vm_map;

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
use crate::device_manager::vm_exit::DeviceVmExitHandler;
use crate::error::Error;
use crate::error::Result;
use crate::virt::SetUserMemoryRegionFlags;
use crate::virt::Virt;
use crate::virt::hvp::irq_chip::HvpGicV3;
use crate::virt::hvp::vcpu::HvpVcpu;

pub(crate) mod vcpu;

mod irq_chip;

macro_rules! hv_unsafe_call {
    ($x:expr) => {{
        let ret = unsafe { $x };
        match ret {
            x if x == hv_error_t::HV_SUCCESS as i32 => Ok(()),
            code => Err(crate::error::Error::ApplevisorError(HypervisorError::from(
                code,
            ))),
        }
    }};
}

pub(crate) use hv_unsafe_call;

impl From<SetUserMemoryRegionFlags> for MemPerms {
    fn from(flags: SetUserMemoryRegionFlags) -> Self {
        match flags {
            SetUserMemoryRegionFlags::ReadWriteExec => MemPerms::ReadWriteExec,
        }
    }
}

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
    num_vcpus: usize,
    psci: Arc<Psci02>,
    cpu_on_receiver: Option<Vec<Receiver<(u64, u64)>>>,
}

impl Virt for Hvp {
    type Arch = AArch64;

    fn new(num_vcpus: usize) -> Result<Self> {
        let vm_config = unsafe { hv_vm_config_create() };
        hv_unsafe_call!(hv_vm_config_set_el2_enabled(vm_config, true))?;
        hv_unsafe_call!(hv_vm_create(vm_config))?;

        let mut cpu_on_receiver = vec![];
        let mut cpu_on_barrier = vec![];
        for _ in 0..num_vcpus {
            let (tx, rx) = mpsc::channel();
            cpu_on_receiver.push(rx);
            cpu_on_barrier.push(tx);
        }

        Ok(Hvp {
            arch: AArch64 {
                layout: AArch64Layout::default(),
            },
            num_vcpus,
            psci: Arc::new(Psci02 { cpu_on_barrier }),
            cpu_on_receiver: Some(cpu_on_receiver),
        })
    }

    fn create_irq_chip(&mut self) -> Result<Arc<dyn InterruptController>> {
        let layout = self.get_layout_mut();

        let gic_config: hv_gic_config_t = unsafe { hv_gic_config_create() };

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
            hv_unsafe_call!(hv_gic_config_set_distributor_base(
                gic_config,
                distributor_base
            ))?;
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
            hv_unsafe_call!(hv_gic_config_set_redistributor_base(
                gic_config,
                redistributor_base
            ))?;
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
            hv_unsafe_call!(hv_gic_config_set_msi_region_base(gic_config, msi_base))?;
            hv_unsafe_call!(hv_gic_config_set_msi_interrupt_range(gic_config, 256, 256))?;
        }

        if msi_base + gic_msi_region_size as u64 > layout.get_ram_base() {
            return Err(Error::InterruptControllerFailed(
                "msi region too large".to_string(),
            ));
        }

        hv_unsafe_call!(hv_gic_create(gic_config))?;

        Ok(Arc::new(HvpGicV3::new(
            distributor_base,
            redistributor_base,
            msi_base,
        )))
    }

    fn set_user_memory_region(
        &mut self,
        userspace_addr: u64,
        guest_phys_addr: u64,
        memory_size: usize,
        flags: SetUserMemoryRegionFlags,
    ) -> Result<()> {
        hv_unsafe_call!(hv_vm_map(
            userspace_addr as _,
            guest_phys_addr,
            memory_size,
            MemPerms::from(flags) as u64,
        ))?;

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

    fn run(&mut self, device_vm_exit_handler: &dyn DeviceVmExitHandler) -> Result<()> {
        thread::scope(|s| {
            let cpu_on_receiver = self.cpu_on_receiver.take().unwrap();

            for (vcpu_id, rx) in (0..self.num_vcpus).zip(cpu_on_receiver.into_iter()) {
                let psci = self.psci.clone();

                let layout = self.get_layout().clone();

                s.spawn(move || -> anyhow::Result<()> {
                    let mut vcpu = 0;
                    let mut exit = null_mut() as *const hv_vcpu_exit_t;
                    hv_unsafe_call!(hv_vcpu_create(&mut vcpu, &mut exit, null_mut()))?;

                    let mut vcpu = HvpVcpu::new(vcpu_id as u64, vcpu, exit);

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
                        let vm_exit_info = vcpu.run(device_vm_exit_handler)?;

                        match handle_vm_exit(
                            &vcpu,
                            vm_exit_info,
                            psci.as_ref(),
                            device_vm_exit_handler,
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
