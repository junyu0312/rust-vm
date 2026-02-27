use std::sync::Arc;
use std::sync::Mutex;

use crate::arch::aarch64::firmware::psci::Psci;
use crate::arch::vm_exit::aarch64::Error;
use crate::vcpu::arch::aarch64::AArch64Vcpu;

pub fn handle_smc(psci: Arc<Mutex<dyn Psci>>, vcpu: &dyn AArch64Vcpu) -> Result<(), Error> {
    let psci = psci.lock().unwrap();

    psci.call(vcpu)
        .map_err(|err| Error::SmcErr(err.to_string()))
}
