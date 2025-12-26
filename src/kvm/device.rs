use std::ffi::CString;
use std::io;

use nix::libc::O_RDWR;
use nix::libc::c_char;
use nix::libc::c_int;
use nix::libc::open;

pub fn open_kvm() -> io::Result<c_int> {
    let kvm = CString::new("/dev/kvm").unwrap();
    let kvm_fd = unsafe { open(kvm.as_ptr() as *const c_char, O_RDWR) };
    if kvm_fd < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(kvm_fd)
}
