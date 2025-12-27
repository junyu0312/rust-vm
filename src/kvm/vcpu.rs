use std::io;

use anyhow::anyhow;
use nix::libc::c_int;

use crate::kvm::ioctl::create_vcpu;
use crate::kvm::ioctl::kvm_run;
use crate::kvm::vm::KvmVm;

#[derive(Debug)]
pub struct KvmVcpu {
    #[allow(dead_code)]
    vcpu_id: c_int,
    vcpu_fd: c_int,
}

#[derive(thiserror::Error, Debug)]
pub enum KvmVcpuRunError {
    #[error("Failed to run vCPU: {0}")]
    Io(#[from] io::Error),
}

fn create_vcpus(vm_fd: c_int, num_vcpus: usize) -> io::Result<Vec<KvmVcpu>> {
    let mut vcpu_fds = Vec::with_capacity(num_vcpus);

    for vcpu_id in 0..num_vcpus {
        let vcpu_id = vcpu_id as c_int;
        let vcpu_fd = create_vcpu(vm_fd, vcpu_id)?;

        vcpu_fds.push(KvmVcpu { vcpu_id, vcpu_fd });
    }

    Ok(vcpu_fds)
}

impl KvmVcpu {
    fn run(&self) -> Result<(), KvmVcpuRunError> {
        let ret = kvm_run(self.vcpu_fd)?;

        assert_eq!(ret, 0);

        Ok(())
    }
}

impl KvmVm {
    pub fn create_vcpus(&mut self, num_vcpus: usize) -> anyhow::Result<()> {
        let vcpus = create_vcpus(self.vm_fd, num_vcpus)?;
        self.vcpus
            .set(vcpus)
            .map_err(|_| anyhow!("vcpus are already set"))?;
        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let vcpus = self
            .vcpus
            .get()
            .ok_or_else(|| anyhow!("vcpus are not created"))?;

        for vcpu in vcpus {
            vcpu.run()?;
        }

        Ok(())
    }
}
