use zerocopy::{Immutable, IntoBytes};

use crate::device::capability::PciCapId;

#[derive(IntoBytes, Immutable)]
#[repr(C, packed)]
pub struct PciMsiCap {
    capability_id: u8,
    next_pointer: u8,
    pub message_control: u16,
    pub message_address: u64,
    pub message_data: u16,
}

#[repr(u8)]
pub enum PciMsiMmc {
    N1 = 0b000,
    N2 = 0b001,
    N4 = 0b010,
    N8 = 0b011,
    N16 = 0b100,
    N32 = 0b101,
}

impl PciMsiCap {
    pub fn new(mmc: PciMsiMmc) -> Self {
        // Enable 2^mmc vectors and enable 64bit address
        let message_control = ((mmc as u16) << 1) | (1 << 7);

        Self {
            capability_id: PciCapId::Msi as u8,
            next_pointer: Default::default(),
            message_control,
            message_address: Default::default(),
            message_data: Default::default(),
        }
    }
}
