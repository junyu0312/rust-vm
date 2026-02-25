use std::sync::mpsc;

use strum_macros::FromRepr;

use crate::firmware::psci::Psci;
use crate::firmware::psci::error::PsciError;
use crate::firmware::psci::function::psci_0_2_fn;
use crate::firmware::psci::function::psci_0_2_fn64;
use crate::firmware::psci::return_value::PsciRet;
use crate::firmware::psci::version::psci_version;
use crate::vcpu::arch::aarch64::AArch64Vcpu;

#[derive(FromRepr)]
#[repr(u32)]
enum Psci02FunctionId {
    Version = psci_0_2_fn(0),
    CpuSuspend = psci_0_2_fn(1),
    CpuSuspend64 = psci_0_2_fn64(1),
    CpuOff = psci_0_2_fn(2),
    CpuOn = psci_0_2_fn(3),
    CpuOn64 = psci_0_2_fn64(3),
}

pub struct Psci02 {
    pub cpu_on_barrier: Vec<mpsc::Sender<(u64, u64)>>,
}

impl Psci for Psci02 {
    fn version(&self) -> u32 {
        psci_version(0, 2)
    }

    fn call(&self, vcpu: &dyn AArch64Vcpu) -> Result<(), PsciError> {
        let function_id = vcpu.get_smc_function_id().unwrap();

        let val = match Psci02FunctionId::from_repr(function_id) {
            Some(function_id) => match function_id {
                Psci02FunctionId::Version => self.version(),
                Psci02FunctionId::CpuSuspend => todo!(),
                Psci02FunctionId::CpuSuspend64 => todo!(),
                Psci02FunctionId::CpuOff => todo!(),
                Psci02FunctionId::CpuOn => {
                    todo!()
                }
                Psci02FunctionId::CpuOn64 => {
                    // mpidr
                    let target_cpu = vcpu.get_smc_arg1().unwrap();
                    let entry_point_address = vcpu.get_smc_arg2().unwrap();
                    let context_id = vcpu.get_smc_arg3().unwrap();

                    self.cpu_on_barrier[target_cpu as usize]
                        .send((entry_point_address, context_id))
                        .unwrap();

                    PsciRet::SUCCESS as u32
                }
            },
            None => PsciRet::NOT_SUPPORTED as u32,
        };

        vcpu.set_smc_return_value(val, 0, 0, 0).unwrap();

        Ok(())
    }
}
