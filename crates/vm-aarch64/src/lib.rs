#[deny(warnings)]
pub mod register;

#[macro_export]
macro_rules! get_field {
    ($v:expr, $hi:expr, $lo:expr) => {
        (($v >> $lo) & ((1u64 << ($hi - $lo + 1)) - 1))
    };
}

#[macro_export]
macro_rules! get_field_bit {
    ($v:expr, $b:expr) => {
        (($v >> $b) & 0x1) != 0
    };
}
