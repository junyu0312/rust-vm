use std::marker::PhantomData;

use strum_macros::FromRepr;

use crate::pci::types::configuration_space::ConfigurationSpace;
use crate::pci::types::configuration_space::type0::Type0Header;
use crate::pci::types::function::PciFunction;
use crate::pci::types::function::PciTypeFunctionCommon;

#[derive(FromRepr)]
#[repr(u16)]
enum Type0HeaderOffset {
    VendorId = 0x00,
    DeviceId = 0x02,
    Command = 0x04,
    Status = 0x06,
    RevisionId = 0x08,
    ProgIf = 0x09,
    Subclass = 0x0A,
    ClassCode = 0x0B,
    CacheLineSize = 0x0C,
    LatencyTimer = 0x0D,
    HeaderType = 0x0E,
    Bist = 0x0F,
    Bar0 = 0x10,
    Bar1 = 0x14,
    Bar2 = 0x18,
    Bar3 = 0x1c,
    Bar4 = 0x20,
    Bar5 = 0x24,
    // TODO: More
    RomAddress = 0x30,
}

pub trait PciType0Function: PciTypeFunctionCommon {
    const BAR_SIZE: [Option<u32>; 6];
}

pub struct Type0Function<T> {
    configuration_space: ConfigurationSpace,
    _mark: PhantomData<T>,
}

impl<T> Default for Type0Function<T>
where
    T: PciType0Function,
{
    fn default() -> Self {
        let configuration_space = T::new_configuration_space(0);

        Type0Function {
            configuration_space,
            _mark: PhantomData,
        }
    }
}

impl<T> PciFunction for Type0Function<T>
where
    T: PciType0Function,
{
    fn write_bar(&mut self, n: usize, buf: &[u8]) {
        let val = u32::from_le_bytes(buf.try_into().unwrap());
        let header = self.configuration_space.as_header_mut::<Type0Header>();

        if let Some(bar_size) = T::BAR_SIZE[n] {
            if val == u32::MAX {
                header.bar[n] = !(bar_size - 1);
            } else {
                header.bar[n] = val;
            }
        } else {
            header.bar[n] = 0;
        }
    }

    fn ecam_read(&self, offset: u16, buf: &mut [u8]) {
        self.configuration_space.read(offset, buf);
    }

    fn ecam_write(&mut self, offset: u16, buf: &[u8]) {
        match Type0HeaderOffset::from_repr(offset) {
            Some(Type0HeaderOffset::Bar0) => self.write_bar(0, buf),
            Some(Type0HeaderOffset::Bar1) => self.write_bar(1, buf),
            Some(Type0HeaderOffset::Bar2) => self.write_bar(2, buf),
            Some(Type0HeaderOffset::Bar3) => self.write_bar(3, buf),
            Some(Type0HeaderOffset::Bar4) => self.write_bar(4, buf),
            Some(Type0HeaderOffset::Bar5) => self.write_bar(5, buf),
            Some(Type0HeaderOffset::RomAddress) => (), // Ignore
            _ => self.configuration_space.write(offset, buf),
        }
    }
}
