use crate::types::device_id::DeviceId;

pub const VENDOR_ID: u16 = 0x1AF4;

#[repr(u16)]
pub enum VirtioPciDeviceId {
    Blk = 0x1040 + DeviceId::Blk as u16,
}
