use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::thread;

use crate::device::i8042::controller_cfg::ControllerConfigurationByte;
use crate::device::i8042::keyboard::KeyboardController;
use crate::device::i8042::status_register::StatusRegister;
use crate::device::irq::InterruptController;
use crate::device::pio::PioDevice;
use crate::utils::keyboard::SCANCODE_SET2_MAP;

mod controller_cfg;
mod keyboard;
mod status_register;

const DATA_PORT: u16 = 0x60;
const REGISTER_PORT: u16 = 0x64;
const COMMAND_REGISTER: u16 = REGISTER_PORT;
const STATUS_REGISTER: u16 = REGISTER_PORT;

const BUFFER_SIZE: usize = 16;

#[allow(dead_code)]
struct I8042Raw {
    kbd_ctl: KeyboardController,
    status_register: StatusRegister,
    ccb: ControllerConfigurationByte,

    output_buffer: VecDeque<u8>, // TODO: use ring?
    pending_command: u8,

    irq: Arc<dyn InterruptController>,
}

impl I8042Raw {
    fn new(irq: Arc<dyn InterruptController>) -> Self {
        I8042Raw {
            kbd_ctl: Default::default(),
            status_register: Default::default(),
            ccb: Default::default(),
            output_buffer: VecDeque::with_capacity(BUFFER_SIZE),
            pending_command: Default::default(),
            irq,
        }
    }

    fn update_state(&mut self) {
        let active;

        if self.output_buffer.is_empty() {
            active = false;
            self.status_register.remove(StatusRegister::OBF);
        } else {
            active = true;
            self.status_register.insert(StatusRegister::OBF);
        }

        self.irq.trigger_irq(1, active);
    }

    fn push_output_buffer(&mut self, val: u8) {
        if self.output_buffer.len() >= BUFFER_SIZE {
            return;
        }

        self.output_buffer.push_back(val);
        self.update_state();
    }

    fn read_data(&mut self, data: &mut [u8]) {
        if let Some(val) = self.output_buffer.pop_front() {
            data[0] = val;
            self.irq.trigger_irq(1, false);
            self.update_state();
        }
    }

    fn read_status_register(&self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        data[0] = self.status_register.bits();
    }

    fn write_data(&mut self, data: u8) {
        match self.pending_command {
            0x60 => {
                self.ccb = ControllerConfigurationByte::from_bits_truncate(data);
                self.update_state();
            }
            0xd3 => {
                // TODO: mouse
            }
            0 => {
                self.push_output_buffer(0xfa);
                self.update_state();
            }
            _ => todo!("{}", self.pending_command),
        }

        self.pending_command = 0;
    }

    fn write_command_reg(&mut self, cmd: u8) {
        match cmd {
            0x20 => self.push_output_buffer(self.ccb.bits()),
            0x60 => self.pending_command = cmd,
            0xd3 => self.pending_command = cmd,
            0xa7 => self
                .ccb
                .insert(ControllerConfigurationByte::SecondPs2PortClock),
            0xa8 => self
                .ccb
                .remove(ControllerConfigurationByte::SecondPs2PortClock),
            0xa9 => self.push_output_buffer(0xff), // TODO: Mouse is not supported yet
            _ => todo!("{cmd}"),
        }
    }

    fn io_in(&mut self, port: u16, data: &mut [u8]) {
        // debug!(port, ?data);
        match port {
            DATA_PORT => self.read_data(data),
            STATUS_REGISTER => self.read_status_register(data),
            _ => unreachable!(),
        }
    }

    fn io_out(&mut self, port: u16, data: &[u8]) {
        let cmd = data[0];
        // debug!(port, ?data);
        match port {
            DATA_PORT => {
                self.write_data(cmd);
            }
            COMMAND_REGISTER => {
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
                                raw.push_output_buffer(b);
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
