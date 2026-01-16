/*
 * https://isdaman.com/alsos/hardware/mouse/ps2interface.htm
 */
use std::collections::VecDeque;

use strum_macros::FromRepr;
use tracing::warn;

use crate::device::i8042::ps2::BUFFER_SIZE;
use crate::device::i8042::ps2::Ps2DeviceT;
use crate::device::i8042::ps2::psmouse::status_register::StatusRegister;

const ACK: u8 = 0xfa;
const DEFAULT_RATE: u8 = 100;
const DEFAULT_RESOLUTION: u8 = 0x02;

mod status_register {
    use bitflags::bitflags;

    bitflags! {
        #[derive(Default)]
        pub struct StatusRegister: u8 {
            const RIGHT_BTN = 1 << 0;
            const MIDDLE_BTN = 1 << 1;
            const LEFT_BTN = 1 << 2;
            const SCALING = 1 << 4;
            const ENABLE = 1 << 5;
            const MODE = 1 << 6;
        }
    }
}

#[derive(FromRepr, Debug)]
#[repr(u8)]
pub enum PsMouseCommand {
    SetScale11 = 0xe6,
    SetScale21 = 0xe7,
    SetRes = 0xe8,
    SetInfo = 0xe9,
    SetStream = 0xea,
    Poll = 0xeb,
    ResetWrap = 0xec,
    SetPoll = 0xf0,
    GetId = 0xf2,
    SetRate = 0xf3,
    Enable = 0xf4,
    Disable = 0xf5,
    ResetDis = 0xf6,
    ResetBat = 0xff,
}

pub struct PsMouse {
    output: VecDeque<u8>,
    status: StatusRegister,
    resolution: u8,
    rate: u8,
    pending_cmd: Option<PsMouseCommand>,
}

impl Default for PsMouse {
    fn default() -> Self {
        Self {
            output: VecDeque::with_capacity(BUFFER_SIZE),
            status: StatusRegister::default(),
            resolution: DEFAULT_RESOLUTION,
            rate: DEFAULT_RATE,
            pending_cmd: None,
        }
    }
}

impl Ps2DeviceT for PsMouse {
    const IRQ: u32 = 12;
    const OUTPUT_BUFFER_SIZE: usize = 16;

    fn output_buffer_is_empty(&self) -> bool {
        self.output.is_empty()
    }

    fn output_buffer_is_full(&self) -> bool {
        self.output.len() == Self::OUTPUT_BUFFER_SIZE
    }

    fn try_push_output_buffer(&mut self, value: u8) -> Result<(), u8> {
        if self.output.len() >= Self::OUTPUT_BUFFER_SIZE {
            return Err(value);
        }

        self.output.push_back(value);

        Ok(())
    }

    fn pop_output_buffer(&mut self) -> Option<u8> {
        self.output.pop_front()
    }
}

impl PsMouse {
    pub fn reset(&mut self) {
        self.status = StatusRegister::default();
        self.resolution = DEFAULT_RESOLUTION;
        self.rate = DEFAULT_RATE;
    }

    pub fn handle_command(&mut self, cmd: u8) {
        self.try_push_output_buffer(ACK).unwrap();

        if let Some(pending_cmd) = self.pending_cmd.take() {
            match pending_cmd {
                PsMouseCommand::SetRes => self.resolution = cmd,
                PsMouseCommand::SetRate => self.rate = cmd,
                _ => unreachable!(),
            }
        } else {
            match PsMouseCommand::from_repr(cmd) {
                Some(cmd) => match cmd {
                    PsMouseCommand::SetScale11 => self.status.remove(StatusRegister::SCALING),
                    PsMouseCommand::SetScale21 => self.status.insert(StatusRegister::SCALING),
                    PsMouseCommand::SetRes => self.pending_cmd = Some(cmd),
                    PsMouseCommand::SetInfo => {
                        self.try_push_output_buffer(self.status.bits()).unwrap();
                        self.try_push_output_buffer(self.resolution).unwrap();
                        self.try_push_output_buffer(self.rate).unwrap();
                    }
                    PsMouseCommand::SetStream => self.status.remove(StatusRegister::MODE),
                    PsMouseCommand::Poll => todo!(),
                    PsMouseCommand::ResetWrap => (),
                    PsMouseCommand::SetPoll => self.status.insert(StatusRegister::MODE),
                    PsMouseCommand::GetId => self.try_push_output_buffer(0x00).unwrap(),
                    PsMouseCommand::SetRate => self.pending_cmd = Some(cmd),
                    PsMouseCommand::ResetDis => self.reset(),
                    PsMouseCommand::Enable => self.status.insert(StatusRegister::ENABLE),
                    PsMouseCommand::Disable => self.status.remove(StatusRegister::ENABLE),
                    PsMouseCommand::ResetBat => self.reset(),
                },
                None => warn!("unknown aux cmd: 0x{:x}", cmd),
            }
        }
    }
}
