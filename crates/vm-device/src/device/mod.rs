pub mod cmos;
pub mod coprocessor;
pub mod dummy;
pub mod gic;
pub mod i8042;
pub mod pic;
pub mod post_debug;
pub mod uart8250;
pub mod vga;
pub mod virtio;

#[cfg(target_arch = "aarch64")]
pub mod pl011;

pub enum Device {
    #[cfg(target_arch = "aarch64")]
    GicV3,
}

impl Device {
    pub fn is_irq_chip(&self) -> bool {
        match self {
            Device::GicV3 => true,
        }
    }
}
