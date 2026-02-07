pub mod cmos;
pub mod coprocessor;
pub mod dummy;
pub mod dummy_pci;
pub mod i8042;
pub mod pic;
pub mod post_debug;
pub mod uart8250;
pub mod vga;
pub mod virtio;

#[cfg(target_arch = "aarch64")]
pub mod pl011;
