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
    Dbgbvr0El1,
    Dbgbcr0El1,
    Dbgwvr0El1,
    Dbgwcr0El1,
    Dbgbvr1El1,
    Dbgbcr1El1,
    Dbgwvr1El1,
    Dbgwcr1El1,
    MdccintEl1,
    MdscrEl1,
    Dbgbvr2El1,
    Dbgbcr2El1,
    Dbgwvr2El1,
    Dbgwcr2El1,
    Dbgbvr3El1,
    Dbgbcr3El1,
    Dbgwvr3El1,
    Dbgwcr3El1,
    Dbgbvr4El1,
    Dbgbcr4El1,
    Dbgwvr4El1,
    Dbgwcr4El1,
    Dbgbvr5El1,
    Dbgbcr5El1,
    Dbgwvr5El1,
    Dbgwcr5El1,
    Dbgbvr6El1,
    Dbgbcr6El1,
    Dbgwvr6El1,
    Dbgwcr6El1,
    Dbgbvr7El1,
    Dbgbcr7El1,
    Dbgwvr7El1,
    Dbgwcr7El1,
    Dbgbvr8El1,
    Dbgbcr8El1,
    Dbgwvr8El1,
    Dbgwcr8El1,
    Dbgbvr9El1,
    Dbgbcr9El1,
    Dbgwvr9El1,
    Dbgwcr9El1,
    Dbgbvr10El1,
    Dbgbcr10El1,
    Dbgwvr10El1,
    Dbgwcr10El1,
    Dbgbvr11El1,
    Dbgbcr11El1,
    Dbgwvr11El1,
    Dbgwcr11El1,
    Dbgbvr12El1,
    Dbgbcr12El1,
    Dbgwvr12El1,
    Dbgwcr12El1,
    Dbgbvr13El1,
    Dbgbcr13El1,
    Dbgwvr13El1,
    Dbgwcr13El1,
    Dbgbvr14El1,
    Dbgbcr14El1,
    Dbgwvr14El1,
    Dbgwcr14El1,
    Dbgbvr15El1,
    Dbgbcr15El1,
    Dbgwvr15El1,
    Dbgwcr15El1,
    MidrEl1,
    MpidrEl1,
    IdAa64pfr0El1,
    IdAa64pfr1El1,
    IdAa64zfr0El1,
    IdAa64smfr0El1,
    IdAa64dfr0El1,
    IdAa64dfr1El1,
    IdAa64isar0El1,
    IdAa64isar1El1,
    IdAa64mmfr0El1,
    IdAa64mmfr1El1,
    IdAa64mmfr2El1,
    SctlrEl1,
    CpacrEl1,
    ActlrEl1,
    SmpriEl1,
    SmcrEl1,
    Ttbr0El1,
    Ttbr1El1,
    TcrEl1,
    ApiakeyloEl1,
    ApiakeyhiEl1,
    ApibkeyloEl1,
    ApibkeyhiEl1,
    ApdakeyloEl1,
    ApdakeyhiEl1,
    ApdbkeyloEl1,
    ApdbkeyhiEl1,
    ApgakeyloEl1,
    ApgakeyhiEl1,
    SpsrEl1,
    ElrEl1,
    SpEl0,
    Afsr0El1,
    Afsr1El1,
    EsrEl1,
    FarEl1,
    ParEl1,
    MairEl1,
    AmairEl1,
    VbarEl1,
    ContextidrEl1,
    TpidrEl1,
    ScxtnumEl1,
    CntkctlEl1,
    CsselrEl1,
    TpidrEl0,
    TpidrroEl0,
    Tpidr2El0,
    ScxtnumEl0,
    CntvCtlEl0,
    CntvCvalEl0,
    SpEl1,
    CntpCtlEl0,
    CntpCvalEl0,
    CntpTvalEl0,
    CnthctlEl2,
    CnthpCtlEl2,
    CnthpCvalEl2,
    CnthpTvalEl2,
    CntvoffEl2,
    CptrEl2,
    ElrEl2,
    EsrEl2,
    FarEl2,
    HcrEl2,
    HpfarEl2,
    MairEl2,
    MdcrEl2,
    SctlrEl2,
    SpsrEl2,
    SpEl2,
    TcrEl2,
    TpidrEl2,
    Ttbr0El2,
    Ttbr1El2,
    VbarEl2,
    VmpidrEl2,
    VpidrEl2,
    VtcrEl2,
    VttbrEl2,

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
