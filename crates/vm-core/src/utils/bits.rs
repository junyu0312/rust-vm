#[macro_export]
macro_rules! get_field {
    ($v:expr, $hi:expr, $lo:expr) => {
        (($v >> $lo) & ((1u64 << ($hi - $lo + 1)) - 1))
    };
}
