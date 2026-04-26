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
