use std::marker::PhantomData;

use crate::pci::types::configuration_space::ConfigurationSpace;
use crate::pci::types::function::PciFunction;
use crate::pci::types::function::PciTypeFunctionCommon;

pub trait PciType1Function: PciTypeFunctionCommon {}

pub struct Type1Function<T> {
    configuration_space: ConfigurationSpace,
    _mark: PhantomData<T>,
}

impl<T> Default for Type1Function<T>
where
    T: PciType1Function,
{
    fn default() -> Self {
        let configuration_space = T::new_configuration_space(1);

        Type1Function {
            configuration_space,
            _mark: PhantomData,
        }
    }
}

impl<T> PciFunction for Type1Function<T> {
    fn write_bar(&mut self, _n: usize, _buf: &[u8]) {
        todo!()
    }

    fn ecam_read(&self, offset: u16, buf: &mut [u8]) {
        self.configuration_space.read(offset, buf);
    }

    fn ecam_write(&mut self, offset: u16, buf: &[u8]) {
        self.configuration_space.write(offset, buf);
    }
}
