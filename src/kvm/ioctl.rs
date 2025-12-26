use std::ffi::CString;
use std::io;

use nix::libc::O_RDWR;
use nix::libc::c_char;
use nix::libc::c_int;
use nix::libc::open;
use tracing::warn;

const KVM_CAP_NR_VCPUS: c_int = 9;
const KVM_CAP_MAX_VCPUS: c_int = 66;
const KVM_CAP_MAX_VCPU_ID: c_int = 128;

mod inner {
    use nix::ioctl_write_int_bad;
    use nix::request_code_none;

    const KVMIO: u8 = 0xAE;

    ioctl_write_int_bad!(kvm_create_vm, request_code_none!(KVMIO, 0x01));
    ioctl_write_int_bad!(kvm_check_extension, request_code_none!(KVMIO, 0x03));
    ioctl_write_int_bad!(kvm_create_vcpu, request_code_none!(KVMIO, 0x41));
    ioctl_write_int_bad!(kvm_run, request_code_none!(KVMIO, 0x80));
}

pub fn open_kvm() -> io::Result<c_int> {
    let kvm = CString::new("/dev/kvm").unwrap();
    let kvm_fd = unsafe { open(kvm.as_ptr() as *const c_char, O_RDWR) };
    if kvm_fd < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(kvm_fd)
}

fn check_extension(kvm_fd: c_int, extension: c_int) -> io::Result<c_int> {
    let ret = unsafe { inner::kvm_check_extension(kvm_fd, extension)? };

    Ok(ret)
}

pub fn kvm_cap_nr_vcpus(kvm_fd: c_int) -> io::Result<usize> {
    let nr_vcpus = check_extension(kvm_fd, KVM_CAP_NR_VCPUS).unwrap_or_else(|err| {
        // Recommended by the Linux documentation
        // `https://docs.kernel.org/virt/kvm/api.html#kvm-create-vcpu`
        warn!(
            ?err,
            "The KVM_CAP_NR_VCPUS capability is not supported, defaulting to 4 vCPUs"
        );
        4
    });

    Ok(nr_vcpus as usize)
}

pub fn kvm_cap_max_vcpus(kvm_fd: c_int) -> io::Result<usize> {
    let max_vcpus = check_extension(kvm_fd, KVM_CAP_MAX_VCPUS)?;

    Ok(max_vcpus as usize)
}

pub fn kvm_cap_max_vcpu_id(kvm_fd: c_int, kvm_cap_max_vcpus: usize) -> io::Result<usize> {
    let max_vcpu_id = check_extension(kvm_fd, KVM_CAP_MAX_VCPU_ID)
        .map(|r| r as usize)
        .unwrap_or_else(|err| {
            warn!(
                ?err,
                "The KVM_CAP_MAX_VCPU_ID capability is not supported, defaulting to {}",
                kvm_cap_max_vcpus
            );
            kvm_cap_max_vcpus
        });

    Ok(max_vcpu_id)
}

pub fn kvm_create_vm(kvm_fd: c_int) -> io::Result<c_int> {
    let vm_fd = unsafe { inner::kvm_create_vm(kvm_fd, 0)? };

    Ok(vm_fd)
}

pub fn create_vcpu(vm_fd: c_int, vcpu_id: c_int) -> io::Result<c_int> {
    let vcpu_fd = unsafe { inner::kvm_create_vcpu(vm_fd, vcpu_id)? };

    Ok(vcpu_fd)
}

pub fn kvm_run(vcpu_fd: c_int) -> io::Result<c_int> {
    let ret = unsafe { inner::kvm_run(vcpu_fd, 0)? };

    Ok(ret)
}
