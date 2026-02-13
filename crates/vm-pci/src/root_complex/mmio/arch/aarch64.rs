use vm_core::irq::Phandle;
use vm_fdt::FdtWriter;

use crate::root_complex::mmio::PciRootComplexMmio;
use crate::types::interrupt::InterruptMapEntryArch;

impl PciRootComplexMmio {
    pub fn generate_device_tree_arch(&self, fdt: &mut FdtWriter) -> Result<(), vm_fdt::Error> {
        fdt.property_u32("msi-parent", Phandle::MSI as u32)?;

        let internal = self.internal.lock().unwrap();

        let mut entries = vec![];
        for (bus_id, bus) in internal.bus.iter().enumerate() {
            for (device_id, device) in bus.devices() {
                for (function_id, function) in device.functions().enumerate() {
                    if let Some(irq_entry) = function.interrupt_map_entry(
                        bus_id.try_into().unwrap(),
                        *device_id,
                        function_id.try_into().unwrap(),
                    ) {
                        entries.extend(irq_entry.to_vec());
                    }
                }
            }
        }

        fdt.property_array_u32("interrupt-map", &entries[..])?;
        if !entries.is_empty() {
            fdt.property_array_u32("interrupt-map-mask", &[0, 0, 0, 7])?;
        }

        Ok(())
    }
}
