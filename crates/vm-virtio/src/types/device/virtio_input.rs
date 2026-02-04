use std::ffi::CString;
use std::ffi::c_char;

use strum_macros::FromRepr;
use tracing::debug;
use tracing::warn;

use crate::transport::Result;
use crate::transport::VirtIoError;
use crate::types::device::Subsystem;
use crate::types::device::virtio_input::linux_evdev::EventTypes;

pub mod linux_evdev;
pub mod virtio_input_event;

pub const VIRTIO_INPUT_EVENTS_Q: usize = 0;
pub const VIRTIO_INPUT_STATUS_Q: usize = 1;

pub const VIRTIO_INPUT_VIRT_QUEUE: u32 = 2; // 0. eventq, 1: statusq

#[derive(Clone, Copy, Default, Debug, FromRepr)]
#[repr(u8)]
enum InputConfigSelect {
    #[default]
    Unset = 0x00,
    IdName = 0x01,
    IdSerial = 0x02,
    IdDevids = 0x03,
    PropBits = 0x10,
    EvBits = 0x11,
    AbsInfo = 0x12,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct VirtioInputAbsinfo {
    min: u32,
    max: u32,
    fuzz: u32,
    flat: u32,
    res: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct VirtioInputDevids {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

#[repr(C)]
union VirtioInputUnion {
    string: [c_char; 128],
    bitmap: [u8; 128],
    abs: VirtioInputAbsinfo,
    ids: VirtioInputDevids,
}

#[repr(C)]
pub struct VirtioInputConfig {
    select: InputConfigSelect,
    subsel: u8,
    size: u8,
    reserved: [u8; 5],
    u: VirtioInputUnion,
}

impl Default for VirtioInputConfig {
    fn default() -> Self {
        Self {
            select: Default::default(),
            subsel: Default::default(),
            size: Default::default(),
            reserved: Default::default(),
            u: VirtioInputUnion { string: [0; 128] },
        }
    }
}

impl VirtioInputConfig {
    fn write_select(&mut self, select: u8) {
        match InputConfigSelect::from_repr(select) {
            Some(select) => self.select = select,
            None => {
                warn!(select, "invalid select");
            }
        };
    }

    fn write_subsel(&mut self, subsel: u8) {
        self.subsel = subsel;
    }

    fn read_size(&self) -> u8 {
        self.size
    }
}

pub trait VirtIOInput {
    const INPUT_PROP: u32;

    fn id_name(&self) -> &str;

    fn serial(&self) -> &str;

    fn bitmap_of_ev(&self, ev: EventTypes) -> Option<&[u8]>;

    fn get_virtio_input_config(&self) -> &VirtioInputConfig;

    fn get_virtio_input_config_mut(&mut self) -> &mut VirtioInputConfig;

    fn update(&mut self) {
        let select;
        let subsel;

        {
            let cfg = self.get_virtio_input_config_mut();

            // A device MUST set the size field to zero if i doesn’t support
            // a given select and subsel combination.
            cfg.size = 0;

            select = cfg.select;
            subsel = cfg.subsel;
        }

        match (select, subsel) {
            (InputConfigSelect::IdName, 0) => {
                let name = CString::new(self.id_name()).unwrap();
                let len = name.count_bytes().min(128); // Strings do not include a NUL terminator.

                let cfg = self.get_virtio_input_config_mut();

                cfg.size = len as u8;
                unsafe {
                    let dst = cfg.u.string.as_mut_ptr() as *mut c_char;
                    core::ptr::write_bytes(dst, 0, 128);
                    core::ptr::copy_nonoverlapping(name.as_ptr(), dst, len);
                }
            }
            (InputConfigSelect::IdSerial, 0) => {
                // TODO: Avoid the clone
                let name = CString::new(self.serial()).unwrap();
                let len = name.count_bytes().min(128); // Strings do not include a NUL terminator.

                let cfg = self.get_virtio_input_config_mut();

                cfg.size = len as u8;
                unsafe {
                    let dst = cfg.u.string.as_mut_ptr() as *mut c_char;
                    core::ptr::write_bytes(dst, 0, 128);
                    core::ptr::copy_nonoverlapping(name.as_ptr(), dst, len);
                }
            }
            (InputConfigSelect::IdDevids, 0) => {
                // Do nothing
                // Keey the size 0, and the Linux kernel will use `BUS_VIRTUAL` as default.
            }
            (InputConfigSelect::PropBits, 0) => {
                let cfg = self.get_virtio_input_config_mut();

                if Self::INPUT_PROP > 0 {
                    cfg.size = 4;
                    let mut bitmap = [0; 128];
                    bitmap[0..4].copy_from_slice(&Self::INPUT_PROP.to_le_bytes());
                    cfg.u.bitmap = bitmap;
                }
            }
            (InputConfigSelect::EvBits, subsel) => match EventTypes::from_repr(subsel) {
                Some(ev) => {
                    if let Some(bitmap) = self.bitmap_of_ev(ev) {
                        // TODO: Avoid the clone
                        let bitmap = bitmap.to_vec();
                        let cfg = self.get_virtio_input_config_mut();

                        cfg.size = bitmap.len().try_into().unwrap();
                        let mut tmp = [0; 128];
                        tmp[0..bitmap.len()].copy_from_slice(&bitmap);
                        cfg.u.bitmap = tmp;
                    }
                }
                None => {
                    warn!(
                        name = self.id_name(),
                        subsel, "Invalid ev, the bitmap will not be updated"
                    );
                }
            },
            (_, _) => todo!("{select:?}"),
        }
    }
}

impl<T> Subsystem for T
where
    T: VirtIOInput,
{
    type DeviceConfiguration = VirtioInputConfig;

    const DEVICE_ID: u32 = 18;

    fn read_device_configuration(&self, offset: usize, len: usize, data: &mut [u8]) -> Result<()> {
        let cfg = self.get_virtio_input_config();

        if offset == 2 {
            // size
            assert_eq!(len, 1);
            assert_eq!(data.len(), 1);

            let size = cfg.read_size();
            data[0] = size;
        } else if offset >= 8 {
            // payload
            assert_eq!(len, data.len());

            unsafe {
                let src = cfg.u.string.as_ptr() as *const u8;
                core::ptr::write_bytes(data.as_mut_ptr(), 0, data.len());
                core::ptr::copy_nonoverlapping(src.add(offset - 8), data.as_mut_ptr(), data.len());
            }
        } else {
            return Err(VirtIoError::DriverReadDeviceConfigurationInvalid);
        }

        debug!(offset, len, ?data, "read device_cfg");

        Ok(())
    }

    fn write_device_configuration(&mut self, offset: usize, len: usize, data: &[u8]) -> Result<()> {
        debug!(offset, len, ?data, "write device_cfg");

        let cfg = self.get_virtio_input_config_mut();

        match offset {
            0 => {
                // select
                assert_eq!(len, 1);
                assert_eq!(data.len(), 1);
                cfg.write_select(data[0]);
            }
            1 => {
                // subsel
                assert_eq!(len, 1);
                assert_eq!(data.len(), 1);
                cfg.write_subsel(data[0]);
            }
            _ => {
                warn!("driver try to write invalid field");

                return Err(VirtIoError::DriverWriteDeviceConfigurationInvalid);
            }
        }

        self.update();

        Ok(())
    }
}
