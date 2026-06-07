use bitflags::bitflags;
use zerocopy::Immutable;
use zerocopy::IntoBytes;

bitflags! {
    pub struct LocalApicFlag: u32 {
        const ENABLED = 1 << 0;
         const ONLINE_CAPABLE = 1 << 0;
    }
}

#[derive(Immutable, IntoBytes)]
#[repr(C, packed)]
pub struct LocalApic {
    r#type: u8,
    length: u8,
    acpi_processor_uid: u8,
    apic_id: u8,
    flags: u32,
}

impl LocalApic {
    pub fn new(cpu_id: u8) -> Self {
        LocalApic {
            r#type: 0,
            length: 8,
            acpi_processor_uid: cpu_id,
            apic_id: cpu_id,
            flags: LocalApicFlag::ENABLED.bits(),
        }
    }
}

#[derive(Immutable, IntoBytes)]
#[repr(C, packed)]
pub struct IoApic {
    r#type: u8,
    length: u8,
    io_apic_id: u8,
    reserved: u8,
    io_apic_address: u32,
    global_system_interrupt_base: u32,
}

impl IoApic {
    pub fn new(io_apic_address: u32) -> Self {
        IoApic {
            r#type: 1,
            length: 12,
            io_apic_id: 0, // We only support one io_apic
            reserved: 0,
            io_apic_address,
            global_system_interrupt_base: 0,
        }
    }
}
