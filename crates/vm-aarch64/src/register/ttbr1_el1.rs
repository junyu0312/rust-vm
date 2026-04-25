use crate::get_field;

pub struct Ttbr1El1(pub u64);

impl From<u64> for Ttbr1El1 {
    fn from(value: u64) -> Self {
        Ttbr1El1(value)
    }
}

impl Ttbr1El1 {
    pub fn baddr(&self) -> u64 {
        get_field!(self.0, 47, 0)
    }
}
