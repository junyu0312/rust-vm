use crate::acpi::r#type::common_header::CommonHeader;

pub struct Xsdt {
    header: CommonHeader,
    entry: Vec<u8>,
}
