use crate::device::pio::IoAddressSpace;

pub mod arch;

pub trait Vcpu {
    fn run(&mut self, device: &mut IoAddressSpace) -> anyhow::Result<()>;
}
