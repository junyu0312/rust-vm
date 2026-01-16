use strum_macros::FromRepr;

#[derive(FromRepr, Debug)]
#[repr(u8)]
pub enum I8042Cmd {
    CtlRctr = 0x20,
    CtlWctr = 0x60,
    CtlTest = 0xaa,

    AuxDisable = 0xa7,
    AuxEnable = 0xa8,
    AuxTest = 0xa9,
    AuxLoop = 0xd3,
    AuxSend = 0xd4,
}
