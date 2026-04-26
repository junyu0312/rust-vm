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
