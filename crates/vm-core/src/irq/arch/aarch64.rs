use vm_fdt::FdtWriter;

use crate::irq::InterruptController;

pub trait AArch64IrqChip: InterruptController {
    fn get_distributor_base(&self) -> anyhow::Result<u64>;

    fn get_distributor_size(&self) -> anyhow::Result<usize>;

    fn get_redistributor_base(&self) -> anyhow::Result<u64>;

    fn get_redistributor_region_size(&self) -> anyhow::Result<usize>;

    fn write_device_tree(&self, fdt: &mut FdtWriter) -> anyhow::Result<u32>;
}
