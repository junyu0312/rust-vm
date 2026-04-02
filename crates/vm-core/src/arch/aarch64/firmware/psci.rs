/*
 * Refer to `ARM Power State Coordination Interface`
 */

use crate::arch::aarch64::firmware::psci::error::PsciError;
use crate::arch::aarch64::vcpu::AArch64Vcpu;

pub mod error;
pub mod function;
pub mod psci_0_2;
pub mod return_value;
pub mod version;

pub trait Psci: Send + Sync {
    fn version(&self) -> u32;

    fn call(&self, vcpu: &mut dyn AArch64Vcpu) -> Result<(), PsciError>;
}
