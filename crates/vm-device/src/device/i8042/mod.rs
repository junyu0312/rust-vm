use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::thread;

use vm_core::irq::InterruptController;

use crate::device::i8042::command::I8042Cmd;
use crate::device::i8042::controller_cfg::ControllerConfigurationByte;
use crate::device::i8042::ps2::Ps2Device;
use crate::device::i8042::ps2::atkbd::AtKbd;
use crate::device::i8042::ps2::psmouse::PsMouse;
use crate::device::i8042::status_register::StatusRegister;
use crate::device::pio::PioDevice;
use crate::utils::keyboard::SCANCODE_SET2_MAP;

mod command;
mod controller_cfg;
mod ps2;
mod status_register;

const OUTPUT_BUFFER_SIZE: usize = 16;

const DATA_PORT: u16 = 0x60;
const REGISTER_PORT: u16 = 0x64;
const COMMAND_REGISTER: u16 = REGISTER_PORT;
const STATUS_REGISTER: u16 = REGISTER_PORT;

struct I8042Raw {
    atkbd: AtKbd<OUTPUT_BUFFER_SIZE>,
    ps_mouse: PsMouse<OUTPUT_BUFFER_SIZE>,
    status_register: StatusRegister,
    ctr: ControllerConfigurationByte,

    pending_command: Option<I8042Cmd>,

    irq: Arc<dyn InterruptController>,
}

impl I8042Raw {
    fn new(irq: Arc<dyn InterruptController>) -> Self {
        I8042Raw {
            atkbd: AtKbd::<OUTPUT_BUFFER_SIZE>::default(),
            ps_mouse: PsMouse::<OUTPUT_BUFFER_SIZE>::default(),
            status_register: Default::default(),
            ctr: Default::default(),
            pending_command: None,
            irq,
        }
    }

    fn update_state(&mut self) {
        let mut kbd_active = false;
        let mut aux_active = false;

        if !self.atkbd.output_buffer_is_empty() {
            self.status_register.insert(StatusRegister::STR_OBF);
            self.status_register.remove(StatusRegister::STR_AUXDATA);

            kbd_active = true;
        } else if !self.ps_mouse.output_buffer_is_empty() {
            self.status_register.insert(StatusRegister::STR_OBF);
            self.status_register.insert(StatusRegister::STR_AUXDATA);

            aux_active = true;
        } else {
            self.status_register.remove(StatusRegister::STR_OBF);
            self.status_register.remove(StatusRegister::STR_AUXDATA);
        }

        self.atkbd.trigger_irq(self.irq.as_ref(), kbd_active);
        self.ps_mouse.trigger_irq(self.irq.as_ref(), aux_active);
    }

    fn push_kbd(&mut self, val: u8) {
        if self.atkbd.output_buffer_is_full() {
            return;
        }

        self.atkbd.try_push_output_buffer(val).unwrap();

        self.update_state();
    }

    fn push_aux(&mut self, val: u8) {
        if self.ps_mouse.output_buffer_is_full() {
            return;
        }

        self.ps_mouse.try_push_output_buffer(val).unwrap();

        self.update_state();
    }

    fn read_data(&mut self, data: &mut [u8]) {
        if self.status_register.contains(StatusRegister::STR_AUXDATA) {
            data[0] = self.ps_mouse.pop_output_buffer().unwrap();
            self.ps_mouse.trigger_irq(self.irq.as_ref(), false);
            self.update_state();
        } else {
            data[0] = self.atkbd.pop_output_buffer().unwrap();
            self.atkbd.trigger_irq(self.irq.as_ref(), false);
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
                I8042Cmd::AuxTest => todo!(),
                I8042Cmd::AuxLoop => self.push_aux(data),
                I8042Cmd::AuxSend => {
                    self.ps_mouse.handle_command(data);
                    self.update_state();
                }
            },
            None => {
                self.atkbd.handle_command(data);
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
            I8042Cmd::AuxTest => todo!(),
            I8042Cmd::AuxLoop => self.pending_command = Some(cmd),
            I8042Cmd::AuxSend => self.pending_command = Some(cmd),
        }
    }

    fn io_in(&mut self, port: u16, data: &mut [u8]) {
        match port {
            DATA_PORT => self.read_data(data),
            STATUS_REGISTER => self.read_status_register(data),
            _ => unreachable!(),
        }
    }

    fn io_out(&mut self, port: u16, data: &[u8]) {
        let cmd = data[0];
        match port {
            DATA_PORT => self.write_data(cmd),
            COMMAND_REGISTER => self.write_command_reg(cmd),
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
