use crate::irq::arch::aarch64::AArch64IrqChip;
use crate::virt::kvm::irq_chip::KvmIRQ;

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

    fn write_device_tree(&self, _fdt: &mut vm_fdt::FdtWriter) -> anyhow::Result<u32> {
        todo!()
    }
}
