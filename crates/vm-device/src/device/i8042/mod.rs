use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::thread;

use crate::device::i8042::command::I8042Cmd;
use crate::device::i8042::command::KbdCommand;
use crate::device::i8042::controller_cfg::ControllerConfigurationByte;
use crate::device::i8042::device::Ps2Device;
use crate::device::i8042::status_register::StatusRegister;
use crate::device::irq::InterruptController;
use crate::device::pio::PioDevice;
use crate::utils::keyboard::SCANCODE_SET2_MAP;

mod command;
mod controller_cfg;
mod device;
mod status_register;

const KBD_IRQ: u32 = 1;
const AUX_IRQ: u32 = 12;

const DATA_PORT: u16 = 0x60;
const REGISTER_PORT: u16 = 0x64;
const COMMAND_REGISTER: u16 = REGISTER_PORT;
const STATUS_REGISTER: u16 = REGISTER_PORT;

struct I8042Raw {
    kbd_ctl: Ps2Device<KBD_IRQ>,
    aux_ctl: Ps2Device<AUX_IRQ>,
    status_register: StatusRegister,
    ctr: ControllerConfigurationByte,

    pending_command: Option<I8042Cmd>,

    irq: Arc<dyn InterruptController>,
}

impl I8042Raw {
    fn new(irq: Arc<dyn InterruptController>) -> Self {
        I8042Raw {
            kbd_ctl: Default::default(),
            aux_ctl: Default::default(),
            status_register: Default::default(),
            ctr: Default::default(),
            pending_command: None,
            irq,
        }
    }

    fn update_state(&mut self) {
        let mut kbd_active = false;
        let mut aux_active = false;

        if !self.kbd_ctl.is_empty() {
            self.status_register.insert(StatusRegister::STR_OBF);
            self.status_register.remove(StatusRegister::STR_AUXDATA);

            kbd_active = true;
        } else if !self.aux_ctl.is_empty() {
            self.status_register.insert(StatusRegister::STR_OBF);
            self.status_register.insert(StatusRegister::STR_AUXDATA);

            aux_active = true;
        } else {
            self.status_register.remove(StatusRegister::STR_OBF);
            self.status_register.remove(StatusRegister::STR_AUXDATA);
        }

        self.kbd_ctl.trigger_irq(self.irq.as_ref(), kbd_active);
        self.aux_ctl.trigger_irq(self.irq.as_ref(), aux_active);
    }

    fn push_kbd(&mut self, val: u8) {
        if self.kbd_ctl.is_full() {
            return;
        }

        self.kbd_ctl.try_push_back(val).unwrap();

        self.update_state();
    }

    fn push_aux(&mut self, val: u8) {
        if self.aux_ctl.is_full() {
            return;
        }

        self.aux_ctl.try_push_back(val).unwrap();

        self.update_state();
    }

    fn read_data(&mut self, data: &mut [u8]) {
        if self.status_register.contains(StatusRegister::STR_AUXDATA) {
            data[0] = self.aux_ctl.pop_front().unwrap();
            self.aux_ctl.trigger_irq(self.irq.as_ref(), false);
            self.update_state();
        } else {
            data[0] = self.kbd_ctl.pop_front().unwrap();
            self.kbd_ctl.trigger_irq(self.irq.as_ref(), false);
            self.update_state();
        }
    }

    fn read_status_register(&self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        data[0] = self.status_register.bits();
    }

    fn write_data(&mut self, data: u8) {
        match self.pending_command.take() {
            Some(cmd) => match cmd {
                I8042Cmd::CtlRctr => todo!(),
                I8042Cmd::CtlWctr => {
                    self.ctr = ControllerConfigurationByte::from_bits_truncate(data);
                    self.update_state();
                }
                I8042Cmd::CtlTest => todo!(),
                I8042Cmd::AuxDisable => todo!(),
                I8042Cmd::AuxEnable => todo!(),
                I8042Cmd::AuxLoop => self.push_aux(data),
            },
            None => {
                match KbdCommand::from_repr(data) {
                    Some(cmd) => match cmd {
                        KbdCommand::SetLeds => {
                            self.push_kbd(0xfa);
                        }
                        KbdCommand::GetId => {
                            self.push_kbd(0xfa);
                            self.push_kbd(0xab);
                            self.push_kbd(0x41); // or 0x83?
                        }
                        KbdCommand::SetRep => {
                            self.push_kbd(0xfa);
                        }
                        KbdCommand::ResetDis => {
                            self.push_kbd(0xfa);
                        }
                    },
                    None => todo!(),
                }
                self.update_state();
            }
        }
    }

    fn write_command_reg(&mut self, cmd: u8) {
        let Some(cmd) = I8042Cmd::from_repr(cmd) else {
            todo!()
        };

        match cmd {
            I8042Cmd::CtlRctr => self.push_kbd(self.ctr.bits()),
            I8042Cmd::CtlWctr => self.pending_command = Some(cmd),
            I8042Cmd::CtlTest => todo!(),
            I8042Cmd::AuxDisable => self.ctr.insert(ControllerConfigurationByte::CTL_AUXDIS),
            I8042Cmd::AuxEnable => self.ctr.remove(ControllerConfigurationByte::CTL_AUXDIS),
            I8042Cmd::AuxLoop => self.pending_command = Some(cmd),
        }

        // match cmd {
        //     0x20 => self.push_output_buffer(self.ccb.bits()),
        //     0x60 => self.pending_command = cmd,
        //     0xd3 => self.pending_command = cmd,
        //     0xa7 => self
        //         .ccb
        //         .insert(ControllerConfigurationByte::SecondPs2PortClock),
        //     0xa8 => self
        //         .ccb
        //         .remove(ControllerConfigurationByte::SecondPs2PortClock),
        //     0xa9 => self.push_output_buffer(0xff), // TODO: Mouse is not supported yet
        //     _ => todo!("{cmd}"),
        // }
    }

    fn io_in(&mut self, port: u16, data: &mut [u8]) {
        match port {
            DATA_PORT => {
                self.read_data(data);
                println!("8042 read data, port: 0x{:x}, data: 0x{:x}", port, data[0]);
            }
            STATUS_REGISTER => {
                self.read_status_register(data);
                println!(
                    "8042 read status, port: 0x{:x}, data: 0x{:x}",
                    port, data[0]
                );
            }
            _ => unreachable!(),
        }
    }

    fn io_out(&mut self, port: u16, data: &[u8]) {
        let cmd = data[0];
        match port {
            DATA_PORT => {
                println!("8042 write data, port: 0x{:x}, data: 0x{:x}", port, data[0]);
                self.write_data(cmd);
            }
            COMMAND_REGISTER => {
                println!(
                    "8042 write command, port: 0x{:x}, data: 0x{:x}",
                    port, data[0]
                );
                self.write_command_reg(cmd);
            }
            _ => unreachable!(),
        }
    }
}

pub struct I8042(Arc<Mutex<I8042Raw>>);

impl I8042 {
    pub fn new(irq: Arc<dyn InterruptController>, rx: Receiver<u8>) -> Self {
        let i8042 = Arc::new(Mutex::new(I8042Raw::new(irq)));

        thread::spawn({
            let raw = i8042.clone();
            move || {
                loop {
                    if let Ok(c) = rx.recv() {
                        let mut raw = raw.lock().unwrap();
                        if let Some(bytes) = SCANCODE_SET2_MAP.get(&c) {
                            for &b in bytes {
                                raw.push_kbd(b);
                            }
                        }
                    }
                }
            }
        });

        I8042(i8042)
    }
}

impl PioDevice for I8042 {
    fn ports(&self) -> &[u16] {
        &[DATA_PORT, REGISTER_PORT]
    }

    fn io_in(&mut self, port: u16, data: &mut [u8]) {
        self.0.lock().unwrap().io_in(port, data);
    }

    fn io_out(&mut self, port: u16, data: &[u8]) {
        self.0.lock().unwrap().io_out(port, data);
    }
}
