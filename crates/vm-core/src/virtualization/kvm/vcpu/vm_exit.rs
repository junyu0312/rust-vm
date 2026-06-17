use kvm_ioctls::VcpuExit;
use tracing::trace;

use crate::cpu::vm_exit::VmExit;
use crate::cpu::vm_exit::VmExitHandlerError;

pub enum VmExitResult {
    Ok,
}

pub fn handle_vm_exit(
    vcpu_exit: VcpuExit<'_>,
    handler: &dyn VmExit,
) -> Result<VmExitResult, VmExitHandlerError> {
    trace!(?vcpu_exit);

    match vcpu_exit {
        VcpuExit::IoOut(port, data) => {
            handler.io_out(port, data)?;
            Ok(VmExitResult::Ok)
        }
        VcpuExit::IoIn(port, data) => {
            handler.io_in(port, data)?;
            Ok(VmExitResult::Ok)
        }
        VcpuExit::MmioRead(addr, buf) => {
            handler.mmio_read(addr, buf)?;
            Ok(VmExitResult::Ok)
        }
        VcpuExit::MmioWrite(addr, buf) => {
            handler.mmio_write(addr, buf)?;
            Ok(VmExitResult::Ok)
        }
        VcpuExit::Unknown => todo!(),
        VcpuExit::Exception => todo!(),
        VcpuExit::Hypercall(..) => todo!(),
        VcpuExit::Debug(..) => todo!(),
        VcpuExit::Hlt => todo!(),
        VcpuExit::IrqWindowOpen => todo!(),
        VcpuExit::Shutdown => todo!(),
        VcpuExit::FailEntry(_, _) => todo!(),
        VcpuExit::Intr => todo!(),
        VcpuExit::SetTpr => todo!(),
        VcpuExit::TprAccess => todo!(),
        VcpuExit::S390Sieic => todo!(),
        VcpuExit::S390Reset => todo!(),
        VcpuExit::Dcr => todo!(),
        VcpuExit::Nmi => todo!(),
        VcpuExit::InternalError => todo!(),
        VcpuExit::Osi => todo!(),
        VcpuExit::PaprHcall => todo!(),
        VcpuExit::S390Ucontrol => todo!(),
        VcpuExit::Watchdog => todo!(),
        VcpuExit::S390Tsch => todo!(),
        VcpuExit::Epr => todo!(),
        VcpuExit::SystemEvent(..) => todo!(),
        VcpuExit::S390Stsi => todo!(),
        VcpuExit::IoapicEoi(_) => todo!(),
        VcpuExit::Hyperv => todo!(),
        VcpuExit::X86Rdmsr(..) => todo!(),
        VcpuExit::X86Wrmsr(..) => todo!(),
        VcpuExit::MemoryFault { .. } => todo!(),
        VcpuExit::Unsupported(_) => todo!(),
    }
}
