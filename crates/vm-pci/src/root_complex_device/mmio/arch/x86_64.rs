use vm_fdt::FdtWriter;

use crate::root_complex_device::mmio::MmioTransport;

impl MmioTransport {
    pub fn generate_device_tree_arch(&self, _fdt: &mut FdtWriter) -> Result<(), vm_fdt::Error> {
        todo!()
    }
}
