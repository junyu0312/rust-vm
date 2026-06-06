use crate::acpi::r#type::common_header::CommonHeader;

/// Multiple APIC Description Table
pub struct Madt {
    header: CommonHeader,
    local_interrupt_controller_address: u32,
    flags: u32,
    interrupt_controllers: Vec<u8>,
}

impl Madt {
    pub fn new() -> Self {
        Madt {
            header: todo!(),
            local_interrupt_controller_address: todo!(),
            flags: todo!(),
            interrupt_controllers: todo!(),
        }
    }
}
