#[cfg(target_arch = "aarch64")]
use gdbstub_arch::aarch64::AArch64 as GdbStubArch;
#[cfg(target_arch = "x86_64")]
use gdbstub_arch::x86::X86_64_SSE as GdbStubArch;

pub mod connection;
pub mod error;

mod event_loop;
mod target;
