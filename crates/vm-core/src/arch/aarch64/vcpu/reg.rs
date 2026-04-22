use strum_macros::FromRepr;

pub mod esr_el2;

#[derive(Debug)]
pub enum CoreRegister {
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X16,
    X17,
    X18,
    X19,
    X20,
    X21,
    X22,
    X23,
    X24,
    X25,
    X26,
    X27,
    X28,
    X29,
    X30,
    SP,
    PC,
    PState,
    Fpcr,
    Fpsr,
}

impl CoreRegister {
    pub fn from_srt(srt: u64) -> Self {
        match srt {
            0 => CoreRegister::X0,
            1 => CoreRegister::X1,
            2 => CoreRegister::X2,
            3 => CoreRegister::X3,
            4 => CoreRegister::X4,
            5 => CoreRegister::X5,
            6 => CoreRegister::X6,
            7 => CoreRegister::X7,
            8 => CoreRegister::X8,
            9 => CoreRegister::X9,
            10 => CoreRegister::X10,
            11 => CoreRegister::X11,
            12 => CoreRegister::X12,
            13 => CoreRegister::X13,
            14 => CoreRegister::X14,
            15 => CoreRegister::X15,
            16 => CoreRegister::X16,
            17 => CoreRegister::X17,
            18 => CoreRegister::X18,
            19 => CoreRegister::X19,
            20 => CoreRegister::X20,
            21 => CoreRegister::X21,
            22 => CoreRegister::X22,
            23 => CoreRegister::X23,
            24 => CoreRegister::X24,
            25 => CoreRegister::X25,
            26 => CoreRegister::X26,
            27 => CoreRegister::X27,
            28 => CoreRegister::X28,
            29 => CoreRegister::X29,
            30 => CoreRegister::X30,
            _ => unimplemented!("{srt}"),
        }
    }
}

#[derive(Debug, FromRepr)]
pub enum FpRegister {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
    V11,
    V12,
    V13,
    V14,
    V15,
    V16,
    V17,
    V18,
    V19,
    V20,
    V21,
    V22,
    V23,
    V24,
    V25,
    V26,
    V27,
    V28,
    V29,
    V30,
    V31,
}

#[derive(Debug)]
pub enum SysRegister {
    MpidrEl1,
    SctlrEl1,
    CnthctlEl2,
    OslarEl1,
    OslsrEl1,
    OsdlrEl1,
}

impl SysRegister {
    pub fn decode(op0: u8, op1: u8, crn: u8, crm: u8, op2: u8) -> Self {
        if op0 == 0b10 {
            /*
             * Refer to `Table D23-1  Instruction encodings for debug System register access in the (op0==0b10) encoding space`
             */
            match (op1, crn, crm, op2) {
                (0b000, 0b0001, 0b0000, 0b100) => SysRegister::OslarEl1,
                (0b000, 0b0001, 0b0001, 0b100) => SysRegister::OslsrEl1,
                (0b000, 0b0001, 0b0011, 0b100) => SysRegister::OsdlrEl1,
                _ => unimplemented!(),
            }
        } else {
            unimplemented!()
        }
    }
}

// mod pstate {
//     use bitflags::bitflags;

//     bitflags! {
//         #[derive(Default)]
//         pub struct PState: u32 {
//             const N = 1 << 31;
//             const Z = 1 << 30;
//             const C = 1 << 29;
//             const V = 1 << 28;

//             const D = 1 << 9;
//             const A = 1 << 8;
//             const I = 1 << 7;
//             const F = 1 << 6;

//             const PAN = 1 << 5;
//             const UAO = 1 << 4;
//             const SS  = 1 << 3;
//             const IL  = 1 << 2;

//             const SP  = 1 << 0;
//         }
//     }
// }
