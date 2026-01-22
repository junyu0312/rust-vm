use vm_fdt::Error;
use vm_fdt::FdtWriter;

pub struct DtbBuilder;

impl DtbBuilder {
    #[allow(warnings)]
    pub fn build_dtb() -> Result<Vec<u8>, Error> {
        let mut fdt = FdtWriter::new()?;

        let root = fdt.begin_node("")?;

        todo!();

        fdt.end_node(root);

        let dtb = fdt.finish()?;
        Ok(dtb)
    }
}
