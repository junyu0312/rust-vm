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
            19 => CoreRegister::X19,
            _ => unimplemented!("{srt}"),
        }
    }
}

#[derive(Debug)]
pub enum SysRegister {
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

pub mod sctlr_el1 {
    use bitflags::bitflags;

    bitflags! {
        pub struct SctlrEl1: u64 {
            const M = 1 << 0;
            const A = 1 << 1;
            const C = 1 << 2;
            const SA = 1 << 3;
            const SA0 = 1 << 4;
            const CP15BEN = 1 << 5;
            const nAA = 1 << 6;
            const ITD = 1 << 7;
            const SED = 1 << 8;
            const UMA = 1 << 9;
            const EnRCTX = 1 << 10;
            const EOS = 1 << 11;
            const I = 1 << 12;
            const EnDB = 1 << 13;
            const DZE = 1 << 14;
            const UCT = 1 << 15;
            const nTWI = 1 << 16;

            // RESERVE

            const nTWE = 1 << 18;
            const WXN = 1 << 19;
            const TSCXT = 1 << 20;
            const IESB = 1 << 21;
            const EIS = 1 << 22;
            const SPAN = 1 << 23;
            const EOE = 1 << 24;
            const EE = 1 << 25;
            const UCI = 1 << 26;
            const EnDA = 1 << 27;
            const nTLSMD = 1 << 28;
            const LSMAOE = 1 << 29;
            const EnIB = 1 << 30;
            const EnIA = 1 << 31;

            // RESERVE

            const BT0 = 1 << 35;
            const BT1 = 1 << 36;
            const ITFSB = 1 << 37;
            const TCF0 = 1 << 38 | 1 << 39;
            const TCF = 1 << 40 | 1 << 41;
            const ATA0 = 1 << 42;
            const ATA = 1 << 43;
            const DSSBS = 1 << 44;
            const TWEDEn = 1 << 45;
            const TWEDEL = 1 << 46 | 1 << 47 | 1 << 48 | 1 << 49;

            // RESERVE

            const EnASR = 1 << 54;
            const EnAS0 = 1 << 55;
            const EnALS = 1 << 56;
            const EPAN = 1 << 57;

            // RESERVE
        }
    }
}

pub mod cnthctl_el2 {
    use bitflags::bitflags;

    bitflags! {
        pub struct CnthctlEl2: u64 {
            const EL0PCTEN = 1 << 0;
            const EL0VCTEN = 1 << 1;
            const EVNTEN = 1 << 2;
            const EVNTDIR = 1 << 3;
            const EVNTI = 1 << 4 | 1 << 5 | 1 << 6 | 1 << 7;
            const EL0VTEN = 1 << 8;
            const EL0PTEN = 1 << 9;
            const EL1PCTEN = 1 << 10;
            const EL1PTEN = 1 << 11;
            const ECV = 1 << 12;
            const EL1TVT = 1 << 13;
            const EL1TVCT = 1 << 14;
            const EL1NVPCT = 1 << 15;
            const EL1NVVCT = 1 << 16;
            const EVNTIS = 1 << 17;
        }
    }
}
