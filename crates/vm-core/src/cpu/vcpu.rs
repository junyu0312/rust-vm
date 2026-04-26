use crate::arch::registers::ArchCoreRegisters;
use crate::arch::registers::ArchRegisters;
use crate::cpu::error::VcpuError;
use crate::virtualization::vcpu::HypervisorVcpu;

pub struct Vcpu {
    vcpu_id: usize,
    vcpu_instance: Box<dyn HypervisorVcpu>,
    booted: bool,
}

impl Vcpu {
    pub fn new(vcpu_id: usize, vcpu_instance: Box<dyn HypervisorVcpu>) -> Self {
        Vcpu {
            vcpu_id,
            vcpu_instance,
            booted: false,
        }
    }

    pub async fn boot_vcpu(
        &mut self,
        pc: u64,
        dtb_or_context_id: u64,
        stop_on_boot: bool,
    ) -> Result<(), VcpuError> {
        #[cfg(target_arch = "aarch64")]
        {
            use crate::arch::registers::aarch64::AArch64Registers;

            let register = self.vcpu_instance.read_reigsters().await?;
            let registers =
                AArch64Registers::boot_registers(self.vcpu_id, dtb_or_context_id, pc, register);
            self.vcpu_instance.write_registers(registers).await?;
        }

        self.booted = true;

        if !stop_on_boot {
            self.resume().await?;
        }

        Ok(())
    }

    pub async fn read_registers(&mut self) -> Result<ArchRegisters, VcpuError> {
        self.vcpu_instance.read_reigsters().await
    }

    pub async fn read_core_registers(&mut self) -> Result<ArchCoreRegisters, VcpuError> {
        self.vcpu_instance.read_core_registers().await
    }

    pub async fn write_core_registers(
        &mut self,
        registers: ArchCoreRegisters,
    ) -> Result<(), VcpuError> {
        self.vcpu_instance.write_core_registers(registers).await
    }

    pub async fn write_registers(&mut self, registers: ArchRegisters) -> Result<(), VcpuError> {
        self.vcpu_instance.write_registers(registers).await
    }

    pub async fn translate_gva_to_gpa(&self, gva: u64) -> Result<Option<u64>, VcpuError> {
        self.vcpu_instance.translate_gva_to_gpa(gva).await
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
