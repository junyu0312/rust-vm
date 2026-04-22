use crate::get_field;

pub struct IdAa64mmfr0El1(u64);

impl IdAa64mmfr0El1 {
    pub fn pa_range(&self) -> u8 {
        get_field!(self.0, 3, 0) as u8
    }
}
