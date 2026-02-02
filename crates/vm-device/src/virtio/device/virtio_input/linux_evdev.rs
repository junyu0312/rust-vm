/*
 * https://docs.kernel.org/input/event-codes.html#input-event-codes
 */

use strum_macros::FromRepr;

pub mod ev_key;
pub mod ev_syn;

#[derive(FromRepr)]
#[repr(u8)]
pub enum EventTypes {
    Syn = 0x00,
    Key = 0x01,
    Rel = 0x02,
    Abs = 0x03,
    Msc = 0x04,
    Sw = 0x05,
    Led = 0x11,
    Snd = 0x12,
    Rep = 0x14,
    Ff = 0x15,
    Pwr = 0x16,
}

pub trait EventCode {
    fn as_u16(&self) -> u16;
}
