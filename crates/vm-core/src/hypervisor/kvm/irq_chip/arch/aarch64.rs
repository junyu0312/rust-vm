use crate::arch::aarch64::irq::AArch64IrqChip;
use crate::hypervisor::kvm::irq_chip::KvmIRQ;

impl AArch64IrqChip for KvmIRQ {
    fn get_distributor_base(&self) -> u64 {
        todo!()
    }

    fn get_distributor_size(&self) -> anyhow::Result<usize> {
        todo!()
    }

    fn get_redistributor_base(&self) -> u64 {
        todo!()
    }

    fn get_redistributor_region_size(&self) -> anyhow::Result<usize> {
        todo!()
    }

    fn get_msi_region_base(&self) -> u64 {
        todo!()
    }

    fn get_msi_region_size(&self) -> anyhow::Result<usize> {
        todo!()
    }
}
