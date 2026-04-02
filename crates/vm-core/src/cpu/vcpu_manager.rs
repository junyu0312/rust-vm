use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::vcpu::setup_cpu;
use crate::cpu::error::VcpuError;
use crate::cpu::vcpu::Vcpu;
use crate::cpu::vm_exit::VmExit;
use crate::error::Error as VmError;
use crate::hypervisor::vm::HypervisorVm;

pub struct VcpuManager {
    vm_instance: Arc<dyn HypervisorVm>,
    vcpus: Vec<Arc<Mutex<Vcpu>>>,
    handlers: Vec<JoinHandle<Result<(), VcpuError>>>,
}

impl VcpuManager {
    pub fn new(vm_instance: Arc<dyn HypervisorVm>) -> Self {
        VcpuManager {
            vm_instance,
            vcpus: Default::default(),
            handlers: Default::default(),
        }
    }

    pub fn create_vcpu(
        &mut self,
        vcpu_id: usize,
        vm_exit_handler: Arc<dyn VmExit>,
    ) -> Result<(), VmError> {
        let vcpu_instance = self.vm_instance.create_vcpu(vcpu_id)?;

        let vcpu = Vcpu {
            vcpu_instance,
            vm_exit_handler,
        };

        self.vcpus.push(Arc::new(Mutex::new(vcpu)));

        Ok(())
    }

    pub fn start_vcpu(&mut self, vcpu_id: usize, start_pc: u64, x0: u64) -> Result<(), VcpuError> {
        let vcpu = self
            .vcpus
            .get(vcpu_id)
            .ok_or(VcpuError::VcpuNotCreated(vcpu_id))?
            .clone();

        let handle = thread::spawn(move || -> Result<(), VcpuError> {
            let mut vcpu = vcpu.lock().unwrap();

            vcpu.vcpu_instance.post_init_within_thread()?;

            #[cfg(target_arch = "aarch64")]
            {
                use crate::arch::aarch64::vcpu::reg::CoreRegister;
                use crate::arch::aarch64::vm_exit::HandleVmExitResult;
                use crate::arch::aarch64::vm_exit::handle_vm_exit;

                setup_cpu(x0, start_pc, vcpu_id, &mut *vcpu.vcpu_instance)?;

                let vm_exit_handler = vcpu.vm_exit_handler.clone();

                loop {
                    let vm_exit_reason = vcpu.vcpu_instance.run()?;

                    match handle_vm_exit(&mut vcpu, vm_exit_reason, vm_exit_handler.as_ref())? {
                        HandleVmExitResult::Continue => (),
                        HandleVmExitResult::NextInstruction => {
                            let pc = vcpu.vcpu_instance.get_core_reg(CoreRegister::PC)?;
                            vcpu.vcpu_instance.set_core_reg(CoreRegister::PC, pc + 4)?;
                        }
                    }
                }
            }

            #[cfg(not(target_arch = "aarch64"))]
            {
                use std::hint::black_box;

                black_box((start_pc, x0));
                todo!()
            }
        });

        self.handlers.push(handle);

        Ok(())
    }
}
