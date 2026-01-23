use std::collections::BTreeMap;
use std::fmt::Debug;

use anyhow::bail;

pub struct Range<K> {
    pub start: K,
    pub len: usize,
}

pub type PortRange = Range<u16>;
pub type MmioRange = Range<u64>;

#[derive(Default)]
pub struct AddressSpace<K: Debug>(BTreeMap<K, (usize, usize)>); // start |-> (len, device_id)

impl<K> AddressSpace<K>
where
    K: Copy + Debug + Ord + Into<u64>,
{
    pub fn is_overlap(&self, start: K, len: usize) -> bool {
        let end = start.into() + len as u64;

        if let Some((&left_start, &(left_len, _))) = self.0.range(..=start).next_back() {
            let left_start = left_start.into();
            let left_end = left_start + left_len as u64;

            if left_end > start.into() {
                return true;
            }
        }

        if let Some((&right_start, &(_, _))) = self.0.range(start..).next() {
            let right_start = right_start.into();

            if end > right_start {
                return true;
            }
        }

        false
    }

    fn try_insert(&mut self, start: K, len: usize, value: usize) -> anyhow::Result<()> {
        if len == 0 {
            bail!("invalid len");
        }

        if self.is_overlap(start, len) {
            bail!("overlap");
        }

        self.0.insert(start, (len, value));

        Ok(())
    }

    pub fn try_get_value_by_key(&self, key: K) -> Option<(K, usize, usize)> {
        let (&start, &(len, value)) = self.0.range(..=key).next_back()?;

        if key.into() - start.into() < len as u64 {
            Some((start, len, value))
        } else {
            None
        }
    }
}

pub trait Device {
    fn ports(&self) -> &[PortRange];
    fn io_in(&mut self, port: u16, data: &mut [u8]);
    fn io_out(&mut self, port: u16, data: &[u8]);
    fn mmio_read(&mut self, _offset: u64, _len: usize, _data: &mut [u8]) {
        unimplemented!()
    }
    fn mmio_write(&mut self, _offset: u64, _len: usize, _data: &[u8]) {
        unimplemented!()
    }
}

#[derive(Default)]
pub struct IoAddressSpace {
    devices: Vec<Box<dyn Device>>,
    port: AddressSpace<u16>,
    mmio: AddressSpace<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("port(start: 0x{0:x}, len: {1}) is already registered")]
    PortIsAlreadyRegistered(u16, usize),
    #[error("mmio(start: 0x{0:x}, len: {1}) is already registered")]
    MmioIsAlreadyRegistered(u64, usize),
    #[error("no device found for port 0x{0:#x}")]
    NoDeviceForPort(u16),
    #[error("no device found for addr 0x{0:#x}")]
    NoDeviceForAddr(u64),
    #[error(
        "mmio out of memory: mmio_start: 0x{mmio_start:x}, mmio_len: {mmio_len}, addr: 0x{addr:x}"
    )]
    MmioOutOfMemory {
        mmio_start: u64,
        mmio_len: usize,
        addr: u64,
    },
}

impl IoAddressSpace {
    pub fn register(
        &mut self,
        device: Box<dyn Device>,
        mmio: Option<(u64, usize)>,
    ) -> Result<(), Error> {
        let id = self.devices.len();
        for port_range in device.ports() {
            if self.port.is_overlap(port_range.start, port_range.len) {
                return Err(Error::PortIsAlreadyRegistered(
                    port_range.start,
                    port_range.len,
                ));
            }
        }

        if let Some((start, len)) = &mmio
            && self.mmio.is_overlap(*start, *len)
        {
            return Err(Error::MmioIsAlreadyRegistered(*start, *len));
        }

        for port_range in device.ports() {
            self.port
                .try_insert(port_range.start, port_range.len, id)
                .unwrap();
        }
        if let Some((start, len)) = &mmio {
            self.mmio.try_insert(*start, *len, id).unwrap();
        }
        self.devices.insert(id, device);

        Ok(())
    }

    fn get_device_by_port_mut(&mut self, port: u16) -> Option<&mut Box<dyn Device>> {
        if let Some((_, _, device_index)) = self.port.try_get_value_by_key(port) {
            return self.devices.get_mut(device_index);
        }
        None
    }

    pub fn io_in(&mut self, port: u16, data: &mut [u8]) -> Result<(), Error> {
        let Some(device) = self.get_device_by_port_mut(port) else {
            return Err(Error::NoDeviceForPort(port));
        };

        device.io_in(port, data);

        Ok(())
    }

    pub fn io_out(&mut self, port: u16, data: &[u8]) -> Result<(), Error> {
        let Some(device) = self.get_device_by_port_mut(port) else {
            return Err(Error::NoDeviceForPort(port));
        };

        device.io_out(port, data);

        Ok(())
    }

    fn get_device_by_addr_mut(&mut self, addr: u64) -> Option<(MmioRange, &mut Box<dyn Device>)> {
        if let Some((start, len, device_index)) = self.mmio.try_get_value_by_key(addr) {
            return Some((
                MmioRange { start, len },
                self.devices.get_mut(device_index).unwrap(),
            ));
        }
        None
    }

    pub fn mmio_read(&mut self, addr: u64, len: usize, data: &mut [u8]) -> Result<(), Error> {
        let Some((range, device)) = self.get_device_by_addr_mut(addr) else {
            return Err(Error::NoDeviceForAddr(addr));
        };

        if addr + len as u64 > range.start + range.len as u64 {
            return Err(Error::MmioOutOfMemory {
                mmio_start: range.start,
                mmio_len: range.len,
                addr,
            });
        }

        let offset = addr - range.start;

        device.mmio_read(offset, len, data);

        Ok(())
    }

    pub fn mmio_write(&mut self, addr: u64, len: usize, data: &[u8]) -> Result<(), Error> {
        let Some((range, device)) = self.get_device_by_addr_mut(addr) else {
            return Err(Error::NoDeviceForAddr(addr));
        };

        if addr + len as u64 > range.start + range.len as u64 {
            return Err(Error::MmioOutOfMemory {
                mmio_start: range.start,
                mmio_len: range.len,
                addr,
            });
        }

        let offset = addr - range.start;

        device.mmio_write(offset, len, data);

        Ok(())
    }
}
