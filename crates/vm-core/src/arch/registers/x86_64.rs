use gdbstub_arch::x86::reg::X86_64CoreRegs;
use serde::Deserialize;
use serde::Serialize;
use vm_firmware::x86_64::gdt::Gdt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct X86_64Segment {
    pub base: u64,
    pub limit: u32,
    pub selector: u16,
    pub r#type: u8,
    pub present: u8,
    pub dpl: u8,
    pub db: u8,
    pub s: u8,
    pub l: u8,
    pub g: u8,
    pub avl: u8,
    pub unusable: u8,
    pub padding: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct X86_64Dtable {
    pub base: u64,
    pub limit: u16,
    pub padding: [u16; 3],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct X86_64CoreRegisters {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct X86_64SRegisters {
    pub cs: X86_64Segment,
    pub ds: X86_64Segment,
    pub es: X86_64Segment,
    pub fs: X86_64Segment,
    pub gs: X86_64Segment,
    pub ss: X86_64Segment,
    pub tr: X86_64Segment,
    pub ldt: X86_64Segment,
    pub gdt: X86_64Dtable,
    pub idt: X86_64Dtable,
    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub cr8: u64,
    pub efer: u64,
    pub apic_base: u64,
    pub interrupt_bitmap: [u64; 4],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct X86_64Registers {
    pub regs: X86_64CoreRegisters,
    pub sregs: X86_64SRegisters,
}

impl X86_64Registers {
    pub fn boot_registers(
        gdt_addr: u64,
        gdt: Gdt<5>,
        kernel_entry: u64,
        boot_params: u64,
        regs: X86_64Registers,
    ) -> Self {
        let boot_cs = gdt.entries.get(2).unwrap().to_kvm_segment(2);
        let boot_ds = gdt.entries.get(3).unwrap().to_kvm_segment(3);

        X86_64Registers {
            regs: X86_64CoreRegisters {
                rbx: 0,
                rsi: boot_params,
                rdi: 0,
                rsp: 0x90000,
                rbp: 0,
                rip: kernel_entry,
                rflags: regs.regs.rflags & !(1u64 << 9), // disable interrupt
                ..regs.regs
            },
            sregs: X86_64SRegisters {
                cs: boot_cs.into(),
                ds: boot_ds.into(),
                es: boot_ds.into(),
                ss: boot_ds.into(),
                gdt: X86_64Dtable {
                    base: gdt_addr,
                    limit: (std::mem::size_of_val(&gdt) as u16) - 1,
                    ..regs.sregs.gdt
                },
                // At entry, the CPU must be in 32-bit protected mode with paging disabled
                cr0: (regs.sregs.cr0 | 0x1) & !(1 << 31),
                ..regs.sregs
            },
        }
    }
}

impl From<X86_64CoreRegs> for X86_64CoreRegisters {
    fn from(regs: X86_64CoreRegs) -> X86_64CoreRegisters {
        X86_64CoreRegisters {
            rax: regs.regs[0],
            rbx: regs.regs[1],
            rcx: regs.regs[2],
            rdx: regs.regs[3],
            rsi: regs.regs[4],
            rdi: regs.regs[5],
            rsp: regs.regs[6],
            rbp: regs.regs[7],
            r8: regs.regs[8],
            r9: regs.regs[9],
            r10: regs.regs[10],
            r11: regs.regs[11],
            r12: regs.regs[12],
            r13: regs.regs[13],
            r14: regs.regs[14],
            r15: regs.regs[15],
            rip: regs.rip,
            rflags: regs.eflags as u64,
        }
    }
}

impl From<X86_64CoreRegisters> for X86_64CoreRegs {
    fn from(_regs: X86_64CoreRegisters) -> X86_64CoreRegs {
        todo!()
    }
}
