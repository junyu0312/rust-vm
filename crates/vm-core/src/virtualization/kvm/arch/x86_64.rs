use kvm_bindings::kvm_dtable;
use kvm_bindings::kvm_regs;
use kvm_bindings::kvm_segment;
use kvm_bindings::kvm_sregs;

use crate::arch::registers::x86_64::X86_64CoreRegisters;
use crate::arch::registers::x86_64::X86_64Dtable;
use crate::arch::registers::x86_64::X86_64Registers;
use crate::arch::registers::x86_64::X86_64SRegisters;
use crate::arch::registers::x86_64::X86_64Segment;
use crate::arch::x86_64::vcpu::X86_64Vcpu;
use crate::virtualization::kvm::vcpu::KvmVcpuInternal;
use crate::virtualization::vcpu::command::VcpuCommand;
use crate::virtualization::vcpu::command::VcpuCommandResponse;
use crate::virtualization::vcpu::error::VcpuError;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

impl From<kvm_segment> for X86_64Segment {
    fn from(seg: kvm_segment) -> Self {
        X86_64Segment {
            base: seg.base,
            limit: seg.limit,
            selector: seg.selector,
            r#type: seg.type_,
            present: seg.present,
            dpl: seg.dpl,
            db: seg.db,
            s: seg.s,
            l: seg.l,
            g: seg.g,
            avl: seg.avl,
            unusable: seg.unusable,
            padding: seg.padding,
        }
    }
}

impl From<X86_64Segment> for kvm_segment {
    fn from(seg: X86_64Segment) -> Self {
        kvm_segment {
            base: seg.base,
            limit: seg.limit,
            selector: seg.selector,
            type_: seg.r#type,
            present: seg.present,
            dpl: seg.dpl,
            db: seg.db,
            s: seg.s,
            l: seg.l,
            g: seg.g,
            avl: seg.avl,
            unusable: seg.unusable,
            padding: seg.padding,
        }
    }
}

impl From<kvm_dtable> for X86_64Dtable {
    fn from(dtable: kvm_dtable) -> Self {
        X86_64Dtable {
            base: dtable.base,
            limit: dtable.limit,
            padding: dtable.padding,
        }
    }
}

impl From<X86_64Dtable> for kvm_dtable {
    fn from(dtable: X86_64Dtable) -> Self {
        kvm_dtable {
            base: dtable.base,
            limit: dtable.limit,
            padding: dtable.padding,
        }
    }
}

impl From<kvm_regs> for X86_64CoreRegisters {
    fn from(regs: kvm_regs) -> Self {
        X86_64CoreRegisters {
            rax: regs.rax,
            rbx: regs.rbx,
            rcx: regs.rcx,
            rdx: regs.rdx,
            rsi: regs.rsi,
            rdi: regs.rdi,
            rsp: regs.rsp,
            rbp: regs.rbp,
            r8: regs.r8,
            r9: regs.r9,
            r10: regs.r10,
            r11: regs.r11,
            r12: regs.r12,
            r13: regs.r13,
            r14: regs.r14,
            r15: regs.r15,
            rip: regs.rip,
            rflags: regs.rflags,
        }
    }
}

impl From<X86_64CoreRegisters> for kvm_regs {
    fn from(regs: X86_64CoreRegisters) -> Self {
        kvm_regs {
            rax: regs.rax,
            rbx: regs.rbx,
            rcx: regs.rcx,
            rdx: regs.rdx,
            rsi: regs.rsi,
            rdi: regs.rdi,
            rsp: regs.rsp,
            rbp: regs.rbp,
            r8: regs.r8,
            r9: regs.r9,
            r10: regs.r10,
            r11: regs.r11,
            r12: regs.r12,
            r13: regs.r13,
            r14: regs.r14,
            r15: regs.r15,
            rip: regs.rip,
            rflags: regs.rflags,
        }
    }
}

impl From<kvm_sregs> for X86_64SRegisters {
    fn from(regs: kvm_sregs) -> Self {
        X86_64SRegisters {
            cs: regs.cs.into(),
            ds: regs.ds.into(),
            es: regs.es.into(),
            fs: regs.fs.into(),
            gs: regs.gs.into(),
            ss: regs.ss.into(),
            tr: regs.tr.into(),
            ldt: regs.ldt.into(),
            gdt: regs.gdt.into(),
            idt: regs.idt.into(),
            cr0: regs.cr0,
            cr2: regs.cr2,
            cr3: regs.cr3,
            cr4: regs.cr4,
            cr8: regs.cr8,
            efer: regs.efer,
            apic_base: regs.apic_base,
            interrupt_bitmap: regs.interrupt_bitmap,
        }
    }
}

impl From<X86_64SRegisters> for kvm_sregs {
    fn from(regs: X86_64SRegisters) -> Self {
        kvm_sregs {
            cs: regs.cs.into(),
            ds: regs.ds.into(),
            es: regs.es.into(),
            fs: regs.fs.into(),
            gs: regs.gs.into(),
            ss: regs.ss.into(),
            tr: regs.tr.into(),
            ldt: regs.ldt.into(),
            gdt: regs.gdt.into(),
            idt: regs.idt.into(),
            cr0: regs.cr0,
            cr2: regs.cr2,
            cr3: regs.cr3,
            cr4: regs.cr4,
            cr8: regs.cr8,
            efer: regs.efer,
            apic_base: regs.apic_base,
            interrupt_bitmap: regs.interrupt_bitmap,
        }
    }
}

impl<'a> X86_64Vcpu for KvmVcpuInternal<'a> {
    fn get_regs(&self) -> Result<X86_64Registers, VcpuError> {
        let regs = self.vcpu_fd.get_regs()?;
        let sregs = self.vcpu_fd.get_sregs()?;

        Ok(X86_64Registers {
            regs: regs.into(),
            sregs: sregs.into(),
        })
    }

    fn set_regs(&mut self, regs: X86_64Registers) -> Result<(), VcpuError> {
        self.vcpu_fd.set_regs(&regs.regs.into())?;
        self.vcpu_fd.set_sregs(&regs.sregs.into())?;

        Ok(())
    }

    fn get_core_regs(&self) -> Result<X86_64CoreRegisters, VcpuError> {
        let regs = self.vcpu_fd.get_regs()?;

        Ok(regs.into())
    }

    fn set_core_regs(&mut self, regs: X86_64CoreRegisters) -> Result<(), VcpuError> {
        self.vcpu_fd.set_regs(&regs.into())?;

        Ok(())
    }

    fn get_sregs(&self) -> Result<X86_64SRegisters, VcpuError> {
        Ok(self.vcpu_fd.get_sregs()?.into())
    }

    fn set_sregs(&self, sregs: X86_64SRegisters) -> Result<(), VcpuError> {
        self.vcpu_fd.set_sregs(&sregs.into())?;

        Ok(())
    }
}

impl<'a> KvmVcpuInternal<'a> {
    pub fn handle_command(
        &mut self,
        is_running: &AtomicBool,
        cmd: VcpuCommand,
    ) -> Result<VcpuCommandResponse, VcpuError> {
        match cmd {
            VcpuCommand::ReadRegisters => {
                let registers = self.get_regs()?;

                Ok(VcpuCommandResponse::Registers(Box::new(registers)))
            }
            VcpuCommand::WriteRegisters(regs) => {
                self.set_regs(*regs)?;

                Ok(VcpuCommandResponse::Empty)
            }
            VcpuCommand::ReadCoreRegisters => {
                let registers = self.get_core_regs()?;

                Ok(VcpuCommandResponse::CoreRegisters(Box::new(registers)))
            }
            VcpuCommand::WriteCoreRegisters(regs) => {
                self.set_core_regs(*regs)?;

                Ok(VcpuCommandResponse::Empty)
            }
            VcpuCommand::Save => todo!(),
            VcpuCommand::Load(_) => todo!(),
            VcpuCommand::TranslateGvaToGpa(_) => todo!(),
            VcpuCommand::Resume => {
                is_running.store(true, Ordering::Release);

                Ok(VcpuCommandResponse::Empty)
            }
        }
    }
}
