use vm_core::irq::InterruptController;

#[derive(Default)]
pub struct GicV3 {}

impl InterruptController for GicV3 {
    fn trigger_irq(&self, _irq_line: u32, _active: bool) {
        todo!()
    }

    fn send_msi(&self, _intid: u32) {
        todo!()
    }

    fn write_device_tree(
        &self,
        _fdt: &mut vm_fdt::FdtWriter,
    ) -> anyhow::Result<vm_core::irq::Phandle> {
        todo!()
    }
}
