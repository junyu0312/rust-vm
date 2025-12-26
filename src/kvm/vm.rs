use std::io;

use nix::libc::c_int;

use crate::kvm::ioctl::kvm_create_vm;

pub fn create_vm(kvm_fd: c_int) -> io::Result<c_int> {
    let vm_fd = unsafe { kvm_create_vm(kvm_fd)? };

    if vm_fd < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(vm_fd)
}
