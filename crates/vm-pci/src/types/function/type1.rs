use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Mutex;

use crate::types::configuration_space::ConfigurationSpace;
use crate::types::function::BarHandler;
use crate::types::function::Callback;
use crate::types::function::PciFunction;
use crate::types::function::PciTypeFunctionCommon;

pub trait PciType1Function: PciTypeFunctionCommon {}

pub struct Type1Function<T> {
    configuration_space: Arc<Mutex<ConfigurationSpace>>,
    _mark: PhantomData<T>,
}

impl<T> Default for Type1Function<T>
where
    T: PciType1Function,
{
    fn default() -> Self {
        let cfg = ConfigurationSpace::init::<T>(1);

        Type1Function {
            configuration_space: Arc::new(Mutex::new(cfg)),
            _mark: PhantomData,
        }
    }
}

impl<T> PciFunction for Type1Function<T> {
    fn write_bar(&self, _n: u8, _buf: &[u8]) -> Callback {
        todo!()
    }

    fn ecam_read(&self, offset: u16, buf: &mut [u8]) {
        self.configuration_space.lock().unwrap().read(offset, buf);
    }

    fn ecam_write(&self, offset: u16, buf: &[u8]) -> Callback {
        // TODO: update bar cb
        self.configuration_space.lock().unwrap().write(offset, buf);
        Callback::Void
    }

    fn bar_handler(&self, _bar: u8) -> Box<dyn BarHandler> {
        todo!()
    }
}
