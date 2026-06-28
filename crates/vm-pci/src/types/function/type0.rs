use strum_macros::FromRepr;

use crate::device::function::type0::Bar;
use crate::device::function::type0::PciType0Function;
use crate::device::function::type0::Type0Function;
use crate::types::bar::PciBarInfo;
use crate::types::bar::address_of_bar;
use crate::types::configuration_space::command::PciCommand;
use crate::types::configuration_space::header::type0::Type0Header;
use crate::types::function::EcamUpdateCallback;
use crate::types::function::EcamUpdateCallbackOps;
use crate::types::function::PciFunction;

mod arch;

#[derive(Debug, FromRepr)]
#[repr(u16)]
pub enum Type0HeaderOffset {
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
    CardbusCisPointer = 0x28,
    SubsystemVendorId = 0x2c,
    SubsystemId = 0x2e,
    RomBaseAddress = 0x30,
    CapPointer = 0x34,
    Reserved = 0x38,
    InterruptLine = 0x3c,
    InterruptPin = 0x3d,
    MinGnt = 0x3e,
    MaxLat = 0x3f,
}

impl<T> Type0Function<T>
where
    T: PciType0Function,
{
    fn write_bar(&self, n: u8, buf: &[u8]) {
        let internal = self.internal.lock().unwrap();
        let bar_info = internal.function.bar_info();

        let val = u32::from_le_bytes(buf.try_into().unwrap());
        let mut configuration_space = internal.configuration_space.lock().unwrap();
        let header = configuration_space.as_header_mut::<Type0Header>();

        if let Some(bar_info) = &bar_info[n as usize] {
            let bar_size: u32 = match bar_info {
                #[cfg(target_arch = "x86_64")]
                PciBarInfo::Pio { len } => *len,
                PciBarInfo::Mmio { len, .. } => *len,
            }
            .try_into()
            .unwrap();

            if val == u32::MAX {
                header.bar[n as usize] = !(bar_size - 1);
            } else {
                header.bar[n as usize] = val;
            }
        } else {
            header.bar[n as usize] = 0;
        }
    }

    fn write_command(&self, command: u16) -> Option<EcamUpdateCallback> {
        let mut callback_ops = vec![];

        let internal = self.internal.lock().unwrap();
        let bar_info = internal.function.bar_info();

        let mut configuration_space = internal.configuration_space.lock().unwrap();
        let header = configuration_space.as_header_mut::<Type0Header>();
        let old_command = header.common.command;
        header.common.command = command;

        let old_command = PciCommand::from_bits_retain(old_command);
        let command = PciCommand::from_bits_retain(command);

        for (i, info) in bar_info.iter().enumerate() {
            let Some(info) = info else {
                continue;
            };

            let bar = header.bar[i];
            let address = address_of_bar(bar);

            match info {
                #[cfg(target_arch = "x86_64")]
                PciBarInfo::Pio { len } => {
                    if command.contains(PciCommand::IO) && !old_command.contains(PciCommand::IO) {
                        callback_ops.push(EcamUpdateCallbackOps::AddPioRouter {
                            bar: i as u8,
                            port: address as u16..address as u16 + *len as u16,
                        });
                    } else if !command.contains(PciCommand::IO)
                        && old_command.contains(PciCommand::IO)
                    {
                        callback_ops.push(EcamUpdateCallbackOps::RemovePioRouter { bar: i as u8 });
                    }
                }
                PciBarInfo::Mmio { len, .. } => {
                    if command.contains(PciCommand::MEMORY)
                        && !old_command.contains(PciCommand::MEMORY)
                    {
                        callback_ops.push(EcamUpdateCallbackOps::AddMmioRouter {
                            bar: i as u8,
                            pci_address_range: address as u64..address as u64 + *len as u64,
                        });
                    } else if !command.contains(PciCommand::MEMORY)
                        && old_command.contains(PciCommand::MEMORY)
                    {
                        callback_ops.push(EcamUpdateCallbackOps::RemoveMmioRouter { bar: i as u8 });
                    }
                }
            }
        }

        Some(EcamUpdateCallback(callback_ops))
    }
}

impl<T> PciFunction for Type0Function<T>
where
    T: PciType0Function,
{
    fn ecam_read(&self, offset: u16, buf: &mut [u8]) {
        self.internal
            .lock()
            .unwrap()
            .configuration_space
            .lock()
            .unwrap()
            .read(offset, buf);
    }

    fn ecam_write(&self, offset: u16, buf: &[u8]) -> Option<EcamUpdateCallback> {
        match Type0HeaderOffset::from_repr(offset) {
            Some(Type0HeaderOffset::Bar0) => self.write_bar(0, buf),
            Some(Type0HeaderOffset::Bar1) => self.write_bar(1, buf),
            Some(Type0HeaderOffset::Bar2) => self.write_bar(2, buf),
            Some(Type0HeaderOffset::Bar3) => self.write_bar(3, buf),
            Some(Type0HeaderOffset::Bar4) => self.write_bar(4, buf),
            Some(Type0HeaderOffset::Bar5) => self.write_bar(5, buf),
            // Some(Type0HeaderOffset::RomAddress) => todo!(),
            Some(Type0HeaderOffset::Command) => {
                let command = u16::from_le_bytes(buf.try_into().unwrap());
                return self.write_command(command);
            }
            _ => {
                let configuration_space = &mut self.internal.lock().unwrap().configuration_space;
                configuration_space.lock().unwrap().write(offset, buf);
            }
        }

        None
    }

    fn bar_read(&self, bar: u8, offset: u64, buf: &mut [u8]) {
        self.internal
            .lock()
            .unwrap()
            .function
            .bar_read(Bar::from_repr(bar).unwrap(), offset, buf);
    }

    fn bar_write(&self, bar: u8, offset: u64, buf: &[u8]) {
        self.internal
            .lock()
            .unwrap()
            .function
            .bar_write(Bar::from_repr(bar).unwrap(), offset, buf);
    }

    fn legacy_irq(&self) -> Option<(u8, u8)> {
        self.internal.lock().unwrap().function.legacy_interrupt()
    }
}
