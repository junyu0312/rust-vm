use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

pub mod common;
pub mod type0;
pub mod type1;

pub struct ConfigurationSpace {
    pub buf: [u8; 4096],
}

impl ConfigurationSpace {
    pub fn new(buf: [u8; 4096]) -> Self {
        ConfigurationSpace { buf }
    }

    pub fn as_header<T>(&self) -> &T
    where
        T: FromBytes + KnownLayout + Immutable,
    {
        T::ref_from_bytes(&self.buf[0..size_of::<T>()]).unwrap()
    }

    pub fn as_header_mut<T>(&mut self) -> &mut T
    where
        T: IntoBytes + FromBytes + KnownLayout + Immutable,
    {
        T::mut_from_bytes(&mut self.buf[0..size_of::<T>()]).unwrap()
    }

    pub fn read(&self, offset: u16, buf: &mut [u8]) {
        buf.copy_from_slice(&self.buf[offset as usize..offset as usize + buf.len()]);
    }

    pub fn write(&mut self, offset: u16, buf: &[u8]) {
        self.buf[offset as usize..offset as usize + buf.len()].copy_from_slice(buf);
    }
}
