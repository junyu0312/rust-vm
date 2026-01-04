pub mod pio;

pub mod cmos;
pub mod coprocessor;
pub mod dummy;
pub mod pic;
pub mod post_debug;
pub mod uart16550;
pub mod vga;

#[cfg(target_arch = "x86_64")]
pub mod i8042;
