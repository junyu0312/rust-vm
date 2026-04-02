use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::firmware::psci::Psci;
#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::vcpu::setup_cpu;
use crate::device_manager::vm_exit::DeviceVmExitHandler;
use crate::error::Error as VmError;
use crate::vcpu::error::VcpuError;
use crate::vcpu::vcpu::Vcpu;
use crate::virt::Vm;

pub struct VcpuManager {
    vm_instance: Arc<dyn Vm>,
    vcpus: Vec<Arc<Mutex<Vcpu>>>,
}

impl VcpuManager {
    pub fn new(vm_instance: Arc<dyn Vm>) -> Self {
        VcpuManager {
            vm_instance,
            vcpus: Default::default(),
        }
    }

    pub fn create_vcpu(
        &mut self,
        vcpu_id: usize,
        device_vm_exit_handler: Arc<dyn DeviceVmExitHandler>,
        #[cfg(target_arch = "aarch64")] psci: Arc<dyn Psci>,
    ) -> Result<(), VmError> {
        let vcpu_instance = self.vm_instance.create_vcpu(vcpu_id)?;

        let vcpu = Vcpu {
            vcpu_instance,
            device_vm_exit_handler,
            #[cfg(target_arch = "aarch64")]
            psci,
        };

        self.vcpus.push(Arc::new(Mutex::new(vcpu)));

        Ok(())
    }

    pub fn start_vcpu(&self, vcpu_id: usize, start_pc: u64, x0: u64) -> Result<(), VcpuError> {
        let vcpu = self
            .vcpus
            .get(vcpu_id)
            .ok_or(VcpuError::VcpuNotCreated(vcpu_id))?
            .clone();

        thread::spawn(move || -> Result<(), VcpuError> {
            let mut vcpu = vcpu.lock().unwrap();

            vcpu.vcpu_instance.post_init_within_thread()?;

            #[cfg(target_arch = "aarch64")]
            {
                setup_cpu(x0, start_pc, vcpu_id, &mut *vcpu.vcpu_instance)?;
            }

            loop {
                let vm_exit_reason = vcpu.vcpu_instance.run()?;

                #[cfg(target_arch = "aarch64")]
                {
                    use crate::arch::aarch64::vcpu::reg::CoreRegister;
                    use crate::arch::aarch64::vm_exit::HandleVmExitResult;

                    match crate::arch::aarch64::vm_exit::handle_vm_exit(&mut *vcpu, vm_exit_reason)?
                    {
                        HandleVmExitResult::Continue => (),
                        HandleVmExitResult::NextInstruction => {
                            let pc = vcpu.vcpu_instance.get_core_reg(CoreRegister::PC)?;
                            vcpu.vcpu_instance.set_core_reg(CoreRegister::PC, pc + 4)?;
                        }
                    }
                }
            }
        });

        Ok(())
    }
}
