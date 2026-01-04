use crate::device::i8042::controller_configuration_byte::ControllerConfigurationByte;
use crate::device::i8042::status_register::StatusRegister;
use crate::device::pio::PioDevice;

/*
 * https://wiki.osdev.org/I8042_PS/2_Controller#PS/2_Controller_IO_Ports
 */
const DATA_PORT: u16 = 0x60;
const REGISTER_PORT: u16 = 0x64;

mod status_register {
    use bitflags::bitflags;

    bitflags! {
        #[derive(Default)]
        pub struct StatusRegister: u8 {
            const OutputBufferStatus = 1 << 0;
            const InputBufferStatus = 1 << 1;
            const SystemFlag = 1 << 2;
            const Command = 1 << 3;
            const Unknown0 = 1 << 4;
            const Unknown1 = 1 << 5;
            const TimeoutError = 1 << 6;
            const ParityError = 1 << 7;

        }
    }
}

mod controller_configuration_byte {
    use bitflags::bitflags;

    bitflags! {
        pub struct ControllerConfigurationByte: u8 {
            const FirstPs2PortInterrupt = 1 << 0;
            const SecondPs2PortInterrupt = 1 << 1;
            const SystemFlag = 1 << 2;
            const Zero0 = 1 << 3;
            const FirstPs2PortClock = 1 << 4;
            const SecondPs2PortClock = 1 << 5;
            const FirstPs2PortTranslation = 1 << 6;
            const Zero1 = 1 << 7;
        }
    }

    impl Default for ControllerConfigurationByte {
        fn default() -> Self {
            ControllerConfigurationByte::SystemFlag | ControllerConfigurationByte::FirstPs2PortClock
        }
    }
}

#[derive(Default)]
struct Ram {
    controller_configuration_byte: ControllerConfigurationByte, // Byte0
}

#[derive(Default)]
pub struct I8042 {
    status_register: StatusRegister,
    ram: Ram,

    last_command: Option<u8>,
    output_buffer: Option<u8>,
}

impl I8042 {
    fn handle_command_0x20(&mut self) {
        self.status_register
            .insert(StatusRegister::OutputBufferStatus);
        self.output_buffer = Some(self.ram.controller_configuration_byte.bits());
    }

    fn handle_command_0x60(&mut self) {
        self.status_register
            .remove(StatusRegister::InputBufferStatus);
    }

    fn handle_command_0xa7(&mut self) {
        self.ram
            .controller_configuration_byte
            .insert(ControllerConfigurationByte::SecondPs2PortClock);
    }

    fn handle_command_0xa8(&mut self) {
        self.ram
            .controller_configuration_byte
            .remove(ControllerConfigurationByte::SecondPs2PortClock);
    }

    fn handle_command_0xa9(&mut self) {
        self.status_register
            .insert(StatusRegister::OutputBufferStatus);
        self.output_buffer = Some(0x00);
    }

    fn handle_command_0xd3(&mut self) {
        self.status_register
            .remove(StatusRegister::InputBufferStatus);
    }

    fn read_data(&mut self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        self.status_register
            .remove(StatusRegister::OutputBufferStatus);
        data[0] = self.output_buffer.take().unwrap_or_default();
    }

    fn write_data(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 1);
        let data = data[0];

        match self.last_command {
            Some(0x60) => {
                self.ram.controller_configuration_byte =
                    ControllerConfigurationByte::from_bits_truncate(data)
            }
            Some(0xd3) => {
                self.status_register
                    .insert(StatusRegister::OutputBufferStatus);
                self.output_buffer = Some(data);
            }
            _ => todo!(),
        }
    }

    fn read_status_register(&mut self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        data[0] = self.status_register.bits();
    }

    fn write_command_register(&mut self, data: &[u8]) {
        assert!(data.len() == 1);

        let command = data[0];

        match command {
            0x20 => self.handle_command_0x20(),
            0x21..=0x3f => todo!(),
            0x60 => self.handle_command_0x60(),
            0x61..=0x7f => todo!(),
            0xa7 => self.handle_command_0xa7(),
            0xa8 => self.handle_command_0xa8(),
            0xa9 => self.handle_command_0xa9(),
            0xaa => todo!(),
            0xab => todo!(),
            0xac => todo!(),
            0xad => todo!(),
            0xae => todo!(),
            0xc0 => todo!(),
            0xc1 => todo!(),
            0xc2 => todo!(),
            0xd0 => todo!(),
            0xd1 => todo!(),
            0xd2 => todo!(),
            0xd3 => self.handle_command_0xd3(),
            0xd4 => todo!(),
            0xf0..=0xff => todo!(),
            _ => panic!(),
        }

        self.last_command = Some(command);
    }
}

impl PioDevice for I8042 {
    fn ports(&self) -> &[u16] {
        &[DATA_PORT, REGISTER_PORT]
    }

    fn io_in(&mut self, port: u16, data: &mut [u8]) {
        match port {
            DATA_PORT => self.read_data(data),
            REGISTER_PORT => self.read_status_register(data),
            _ => unreachable!(),
        }
    }

    fn io_out(&mut self, port: u16, data: &[u8]) {
        match port {
            DATA_PORT => self.write_data(data),
            REGISTER_PORT => self.write_command_register(data),
            _ => unreachable!(),
        }
    }
}
