use std::sync::Arc;
use std::sync::Mutex;

use strum_macros::FromRepr;
use vm_core::device::mmio::MmioRange;

use crate::types::configuration_space::ConfigurationSpace;
use crate::types::configuration_space::header::type0::Type0Header;
use crate::types::function::BarHandler;
use crate::types::function::Callback;
use crate::types::function::PciFunction;
use crate::types::function::PciTypeFunctionCommon;

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

pub struct Type0Function<T> {
    configuration_space: Arc<Mutex<ConfigurationSpace>>,
    device: T,
}

impl<T> Type0Function<T>
where
    T: PciType0Function,
{
    pub fn new(device: T) -> Self {
        let mut cfg = ConfigurationSpace::init::<T>(0);

        let header = cfg.as_header_mut::<Type0Header>();
        header.interrupt_line = T::IRQ_LINE;
        header.interrupt_pin = T::IRQ_PIN;

        Type0Function {
            configuration_space: Arc::new(Mutex::new(cfg)),
            device,
        }
    }
}

impl<T> PciFunction for Type0Function<T>
where
    T: PciType0Function,
{
    fn write_bar(&self, n: u8, buf: &[u8]) -> Callback {
        let mut configuration_space = self.configuration_space.lock().unwrap();

        let val = u32::from_le_bytes(buf.try_into().unwrap());
        let header = configuration_space.as_header_mut::<Type0Header>();

        if let Some(bar_size) = T::BAR_SIZE[n as usize] {
            if val == u32::MAX {
                header.bar[n as usize] = !(bar_size - 1);
                Callback::Void
            } else {
                println!("{} {}", n, val);
                header.bar[n as usize] = val;
                Callback::RegisterBarClosure((
                    n,
                    MmioRange {
                        start: val as u64,
                        len: bar_size as usize,
                    },
                    self.bar_handler(n).unwrap(),
                ))
            }
        } else {
            header.bar[n as usize] = 0;
            Callback::Void
        }
    }

    fn ecam_read(&self, offset: u16, buf: &mut [u8]) {
        self.configuration_space.lock().unwrap().read(offset, buf);
    }

    fn ecam_write(&self, offset: u16, buf: &[u8]) -> Callback {
        match Type0HeaderOffset::from_repr(offset) {
            Some(Type0HeaderOffset::Bar0) => self.write_bar(0, buf),
            Some(Type0HeaderOffset::Bar1) => self.write_bar(1, buf),
            Some(Type0HeaderOffset::Bar2) => self.write_bar(2, buf),
            Some(Type0HeaderOffset::Bar3) => self.write_bar(3, buf),
            Some(Type0HeaderOffset::Bar4) => self.write_bar(4, buf),
            Some(Type0HeaderOffset::Bar5) => self.write_bar(5, buf),
            Some(Type0HeaderOffset::RomAddress) => Callback::Void,
            _ => {
                let mut configuration_space = self.configuration_space.lock().unwrap();
                configuration_space.write(offset, buf);
                Callback::Void
            }
        }
    }

    fn bar_handler(&self, n: u8) -> Option<Box<dyn BarHandler>> {
        let bar = Bar::from_repr(n).unwrap();
        self.device.bar_handler(bar)
    }
}
