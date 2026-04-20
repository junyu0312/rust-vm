use crate::cpu::error::VcpuError;
use crate::virtualization::vcpu::HypervisorVcpu;

pub struct Vcpu {
    pub vcpu_id: usize,
    pub vcpu_instance: Box<dyn HypervisorVcpu>,
    pub booted: bool,
}

impl Vcpu {
    pub async fn boot_vcpu(
        &mut self,
        pc: u64,
        dtb_or_context_id: u64,
        stop_on_boot: bool,
    ) -> Result<(), VcpuError> {
        #[cfg(target_arch = "aarch64")]
        {
            use crate::arch::aarch64::register::AArch64Registers;

            let register = self.vcpu_instance.read_reigsters().await?;
            let registers = AArch64Registers::boot_registers(
                self.vcpu_id,
                dtb_or_context_id,
                pc,
                register.pstate,
                register.sctlr_el1,
                register.cnthctl_el2,
            );
            self.vcpu_instance.write_registers(registers).await?;
        }

        self.booted = true;

        if !stop_on_boot {
            self.resume().await?;
        }

        Ok(())
    }

    pub async fn get_registers(&mut self) -> Result<(), VcpuError> {
        todo!()
    }

    pub async fn write_registers(&mut self) -> Result<(), VcpuError> {
        todo!()
    }

    pub async fn resume(&mut self) -> Result<(), VcpuError> {
        if !self.booted {
            return Ok(());
        }

        self.vcpu_instance.resume().await
    }

    pub async fn pause(&mut self) -> Result<(), VcpuError> {
        if !self.booted {
            return Ok(());
        }

        self.vcpu_instance.pause().await
    }
}
