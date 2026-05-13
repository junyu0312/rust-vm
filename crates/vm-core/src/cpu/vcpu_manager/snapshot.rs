use std::io::Read;
use std::io::Write;

use vm_snapshot::ops::Snapshotable;

use crate::cpu::vcpu_manager::VcpuManager;
use crate::virtualization::vcpu::error::VcpuError;

impl Snapshotable for VcpuManager {
    type Error = VcpuError;

    fn save(&self, writer: &mut dyn Write) -> Result<(), Self::Error> {
        let vcpu_count = self.vcpus.len() as u64;
        writer
            .write_all(&vcpu_count.to_le_bytes())
            .map_err(|err| VcpuError::Save(Box::new(err)))?;

        for vcpu in &self.vcpus {
            vcpu.save(writer)?;
        }

        Ok(())
    }

    fn restore(&mut self, _reader: &mut dyn Read) -> Result<(), Self::Error> {
        todo!()
    }
}
