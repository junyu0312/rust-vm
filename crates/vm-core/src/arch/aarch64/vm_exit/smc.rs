use std::sync::Arc;
use std::sync::Mutex;

use crate::arch::aarch64::firmware::psci::Psci;
use crate::arch::aarch64::vcpu::AArch64Vcpu;
use crate::arch::aarch64::vm_exit::Error;

pub fn handle_smc(psci: Arc<Mutex<dyn Psci>>, vcpu: &dyn AArch64Vcpu) -> Result<(), Error> {
    let psci = psci.lock().unwrap();

    psci.call(vcpu)
        .map_err(|err| Error::SmcErr(err.to_string()))
}
