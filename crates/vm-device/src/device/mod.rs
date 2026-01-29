pub mod cmos;
pub mod coprocessor;
pub mod dummy;
pub mod i8042;
pub mod pic;
pub mod post_debug;
pub mod uart8250;
pub mod vga;

#[cfg(target_arch = "aarch64")]
pub mod pl011;
