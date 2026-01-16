use std::collections::VecDeque;

use strum_macros::FromRepr;

use crate::device::i8042::ps2::Ps2Device;

const ACK: u8 = 0xfa;

#[derive(FromRepr, Debug)]
#[repr(u8)]
enum KbdCommand {
    SetLeds = 0xed,
    GetId = 0xf2,
    SetRep = 0xf3,
    ResetDis = 0xf5,
}

#[derive(Default)]
pub struct AtKbd<const OUTPUT_BUFFER_SIZE: usize> {
    output: VecDeque<u8>,
}

impl<const OUTPUT_BUFFER_SIZE: usize> Ps2Device for AtKbd<OUTPUT_BUFFER_SIZE> {
    const IRQ: u32 = 1;

    fn output_buffer_is_empty(&self) -> bool {
        self.output.is_empty()
    }

    fn output_buffer_is_full(&self) -> bool {
        self.output.len() == OUTPUT_BUFFER_SIZE
    }

    fn try_push_output_buffer(&mut self, value: u8) -> Result<(), u8> {
        if self.output.len() >= OUTPUT_BUFFER_SIZE {
            return Err(value);
        }

        self.output.push_back(value);

        Ok(())
    }

    fn pop_output_buffer(&mut self) -> Option<u8> {
        self.output.pop_front()
    }

    fn handle_command(&mut self, cmd: u8) {
        self.try_push_output_buffer(ACK).unwrap();
        if let Some(cmd) = KbdCommand::from_repr(cmd) {
            match cmd {
                KbdCommand::SetLeds => (),
                KbdCommand::GetId => {
                    self.try_push_output_buffer(0xab).unwrap();
                    self.try_push_output_buffer(0x41).unwrap(); // or 0x83?
                }
                KbdCommand::SetRep => (),
                KbdCommand::ResetDis => (),
            }
        }
    }
}
