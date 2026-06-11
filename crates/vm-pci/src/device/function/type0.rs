use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

use strum_macros::FromRepr;
use vm_core::device::error::DeviceSnapshotError;

use crate::device::function::PciTypeFunctionCommon;
use crate::error::Error;
use crate::types::configuration_space::ConfigurationSpace;
use crate::types::configuration_space::header::type0::Type0Header;

#[derive(FromRepr)]
#[repr(u8)]
pub enum Bar {
    Bar0 = 0,
    Bar1 = 1,
    Bar2 = 2,
    Bar3 = 3,
    Bar4 = 4,
    Bar5 = 5,
}

pub trait PciType0Function: PciTypeFunctionCommon {
    fn bar_size(&self) -> [Option<u32>; 6];

    fn bar_read(&self, bar: Bar, offset: u64, buf: &mut [u8]);

    fn bar_write(&self, bar: Bar, offset: u64, buf: &[u8]);

    fn pause(&self) -> Result<(), DeviceSnapshotError>;

    fn resume(&self) -> Result<(), DeviceSnapshotError>;

    fn save(&self, writer: &mut dyn Write) -> Result<(), DeviceSnapshotError>;

    fn load(&mut self, reader: &mut dyn Read) -> Result<(), DeviceSnapshotError>;
}

pub(crate) struct Type0FunctionInternal<T> {
    pub(crate) configuration_space: ConfigurationSpace,
    pub(crate) function: T,
}

pub struct Type0Function<T> {
    pub(crate) internal: Arc<Mutex<Type0FunctionInternal<T>>>,
}

impl<T> Type0Function<T>
where
    T: PciType0Function,
{
    pub fn new(function: T) -> Result<Self, Error> {
        let mut configuration_space = ConfigurationSpace::default();
        configuration_space.init(&function, 0);
        function.init_capability(&mut configuration_space)?;

        let header = configuration_space.as_header_mut::<Type0Header>();
        if let Some((irq_line, irq_pin)) = function.legacy_interrupt() {
            header.interrupt_line = irq_line;
            header.interrupt_pin = irq_pin;
        } else {
            header.interrupt_line = 0xff;
            header.interrupt_pin = 0x00;
        }

        let function = Type0Function {
            internal: Arc::new(Mutex::new(Type0FunctionInternal {
                configuration_space,
                function,
            })),
        };

        Ok(function)
    }

    pub fn pause(&self) -> Result<(), DeviceSnapshotError> {
        todo!()
    }

    pub fn resume(&self) -> Result<(), DeviceSnapshotError> {
        todo!()
    }

    pub fn save(&self, writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        let dev = self.internal.lock().unwrap();

        writer.write_all(&dev.configuration_space.buf)?;
        dev.function.save(writer)?;

        Ok(())
    }

    pub fn load(&mut self, reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        let mut dev = self.internal.lock().unwrap();

        reader.read_exact(&mut dev.configuration_space.buf)?;
        dev.function.load(reader)?;

        Ok(())
    }
}
