use crate::acpi::r#type::common_header::CommonHeader;

pub struct Dsdt {
    header: CommonHeader,
    definition_block: Vec<u8>,
}
