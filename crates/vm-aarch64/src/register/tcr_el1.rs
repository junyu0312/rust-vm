use crate::get_field;
use crate::get_field_bit;

pub struct TcrEl1(pub u64);

impl From<u64> for TcrEl1 {
    fn from(v: u64) -> Self {
        TcrEl1(v)
    }
}

impl TcrEl1 {
    pub fn ds(&self) -> bool {
        get_field_bit!(self.0, 59)
    }

    pub fn ips(&self) -> u8 {
        get_field!(self.0, 34, 32) as u8
    }

    pub fn tg1(&self) -> u8 {
        get_field!(self.0, 31, 30) as u8
    }

    pub fn t1sz(&self) -> u8 {
        get_field!(self.0, 21, 16) as u8
    }
}
