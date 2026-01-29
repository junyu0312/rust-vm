use vm_fdt::FdtWriter;

use crate::irq::InterruptController;

pub const GIC_SPI: u32 = 0;
pub const IRQ_TYPE_LEVEL_HIGH: u32 = 0x04;

pub trait AArch64IrqChip: InterruptController {
    fn get_distributor_base(&self) -> anyhow::Result<u64>;

    fn get_distributor_size(&self) -> anyhow::Result<usize>;

    fn get_redistributor_base(&self) -> anyhow::Result<u64>;

    fn get_redistributor_region_size(&self) -> anyhow::Result<usize>;

    fn write_device_tree(&self, fdt: &mut FdtWriter) -> anyhow::Result<u32>;
}
