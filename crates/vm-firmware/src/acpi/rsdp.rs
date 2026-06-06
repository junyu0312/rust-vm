pub struct Rsdp {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_addr: u32,
    length: u32,
    xsdt_addr: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

impl Rsdp {
    pub fn new() -> Rsdp {
        Rsdp {
            signature: todo!(),
            checksum: todo!(),
            oem_id: todo!(),
            revision: todo!(),
            rsdt_addr: todo!(),
            length: todo!(),
            xsdt_addr: todo!(),
            extended_checksum: todo!(),
            reserved: todo!(),
        }
    }
}
