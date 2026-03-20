pub mod mmio;
pub mod pio;

pub trait Device: Send + Sync {
    fn name(&self) -> String;
}
