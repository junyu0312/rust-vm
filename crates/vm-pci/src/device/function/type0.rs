use std::sync::Arc;
use std::sync::Mutex;

use strum_macros::FromRepr;

use crate::device::function::BarHandler;
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
    const BAR_SIZE: [Option<u32>; 6];

    fn bar_handler(&self, bar: Bar) -> Option<Box<dyn BarHandler>>;
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
        let mut configuration_space = ConfigurationSpace::new();
        configuration_space.init::<T>(0);
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
}
