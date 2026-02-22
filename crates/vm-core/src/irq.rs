use vm_fdt::FdtWriter;

pub mod arch;

#[repr(u32)]
pub enum Phandle {
    GIC = 0x1,
    MSI = 0x2,
}

pub trait InterruptController: Send + Sync + 'static {
    fn trigger_irq(&self, irq_line: u32, active: bool);

    fn send_msi(&self, intid: u32);

    fn write_device_tree(&self, fdt: &mut FdtWriter) -> anyhow::Result<Phandle>;
}
