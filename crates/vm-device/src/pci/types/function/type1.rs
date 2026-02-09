use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Mutex;

use vm_core::device::mmio::mmio_device::MmioHandler;

use crate::pci::types::configuration_space::ConfigurationSpace;
use crate::pci::types::function::Callback;
use crate::pci::types::function::PciFunction;
use crate::pci::types::function::PciTypeFunctionCommon;

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
        let configuration_space = Arc::new(Mutex::new(T::new_configuration_space(1)));

        Type1Function {
            configuration_space,
            _mark: PhantomData,
        }
    }
}

impl<T> PciFunction for Type1Function<T> {
    fn write_bar(&self, pci_address_to_gpa: u64, _n: u8, _buf: &[u8]) -> Callback {
        todo!()
    }

    fn ecam_read(&self, offset: u16, buf: &mut [u8]) {
        self.configuration_space.lock().unwrap().read(offset, buf);
    }

    fn ecam_write(&self, pci_address_to_gpa: u64, offset: u16, buf: &[u8]) -> Callback {
        // TODO: update bar cb
        self.configuration_space.lock().unwrap().write(offset, buf);
        Callback::Void
    }

    fn bar_handler(&self, bar: u8, gpa: u64) -> Box<dyn MmioHandler> {
        todo!()
    }
}
