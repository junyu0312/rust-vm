use std::slice::Iter;

use vm_core::device::Device;

pub fn build_definition_block(devices: Iter<'_, Box<dyn Device>>) -> Vec<u8> {
    let mut definition_block = vec![];
    for device in devices {
        if let Some(aml) = device.support_aml() {
            aml.to_aml_bytes(&mut definition_block);
        }
    }
    definition_block
}
