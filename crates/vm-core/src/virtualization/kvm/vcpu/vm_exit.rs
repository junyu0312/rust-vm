use kvm_ioctls::VcpuExit;
use tracing::debug;

pub fn handle_vm_exit(vcpu_exit: VcpuExit<'_>) {
    debug!(?vcpu_exit);

    match vcpu_exit {
        VcpuExit::IoOut(..) => todo!(),
        VcpuExit::IoIn(..) => todo!(),
        VcpuExit::MmioRead(..) => todo!(),
        VcpuExit::MmioWrite(..) => todo!(),
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
