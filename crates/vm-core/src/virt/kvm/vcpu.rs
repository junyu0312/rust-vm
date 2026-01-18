use kvm_ioctls::*;

mod arch;

#[derive(Debug)]
pub struct KvmVcpu {
    pub vcpu_id: u64,
    pub vcpu_fd: VcpuFd,
}
