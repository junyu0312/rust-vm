use crate::get_field;

pub struct IdAa64mmfr0El1(pub u64);

impl From<u64> for IdAa64mmfr0El1 {
    fn from(value: u64) -> Self {
        IdAa64mmfr0El1(value)
    }
}

impl IdAa64mmfr0El1 {
    pub fn pa_range(&self) -> u8 {
        get_field!(self.0, 3, 0) as u8
    }
}
