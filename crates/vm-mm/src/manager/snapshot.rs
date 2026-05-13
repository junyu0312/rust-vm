use std::io::Read;
use std::io::Write;

use async_trait::async_trait;
use vm_snapshot::ops::Snapshotable;

use crate::error::Error;
use crate::manager::MemoryAddressSpace;

#[async_trait]
impl Snapshotable for MemoryAddressSpace {
    type Error = Error;

    fn save(&self, writer: &mut dyn Write) -> Result<(), Error> {
        let len = self.regions.len() as u64;
        writer
            .write_all(&len.to_le_bytes())
            .map_err(|err| Error::Save(Box::new(err)))?;

        for (gpa, region) in &self.regions {
            writer
                .write_all(&gpa.to_le_bytes())
                .map_err(|err| Error::Save(Box::new(err)))?;

            let region_len = region.len() as u64;
            writer
                .write_all(&region_len.to_le_bytes())
                .map_err(|err| Error::Save(Box::new(err)))?;

            let hva = region.hva();
            unsafe {
                let slice = std::slice::from_raw_parts(hva, region.len());
                writer
                    .write_all(slice)
                    .map_err(|err| Error::Save(Box::new(err)))?;
            }
        }

        Ok(())
    }

    fn restore(&mut self, _reader: &mut dyn Read) -> Result<(), Error> {
        todo!()
    }
}
