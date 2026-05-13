use std::io::Read;
use std::io::Write;

use vm_snapshot::ops::Snapshotable;

use crate::cpu::vcpu_manager::VcpuManager;

impl Snapshotable for VcpuManager {
    fn save(&self, writer: &mut dyn Write) -> Result<(), vm_snapshot::ops::Error> {
        let vcpu_count = self.vcpus.len() as u64;
        writer.write_all(&vcpu_count.to_le_bytes())?;

        for vcpu in &self.vcpus {
            vcpu.save(writer)?;
        }

        Ok(())
    }

    fn restore(&mut self, _reader: &mut dyn Read) -> Result<(), vm_snapshot::ops::Error> {
        todo!()
    }
}
