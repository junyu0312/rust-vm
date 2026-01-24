use anyhow::anyhow;
use strum_macros::FromRepr;

pub struct EsrEl2(u64);

impl From<u64> for EsrEl2 {
    fn from(value: u64) -> Self {
        EsrEl2(value)
    }
}

#[derive(FromRepr)]
#[repr(u8)]
pub enum Ec {
    Unknown = 0x00,
    DA = 0x24,
}

impl EsrEl2 {
    pub fn iss(&self) -> u64 {
        self.0 & 0x1ffffff
    }

    pub fn il(&self) -> bool {
        (self.0 >> 25) & 0x1 != 0
    }

    pub fn ec_raw(&self) -> u8 {
        ((self.0 >> 26) & 0x3f) as u8
    }

    pub fn ec(&self) -> anyhow::Result<Ec> {
        Ec::from_repr(self.ec_raw()).ok_or(anyhow!(format!("unknown ec: 0x{:x}", self.ec_raw())))
    }

    pub fn iss2(&self) -> u64 {
        (self.0 >> 32) & 0xffffff
    }
}
