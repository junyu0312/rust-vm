/*
 * Refer to `ARM Power State Coordination Interface`
 */

use crate::firmware::psci::error::PsciError;
use crate::vcpu::arch::aarch64::AArch64Vcpu;

pub mod error;
pub mod function;
pub mod psci_0_2;
pub mod return_value;
pub mod version;

pub trait Psci {
    fn version(&self) -> u32;

    fn call(&self, vcpu: &dyn AArch64Vcpu) -> Result<(), PsciError>;
}
