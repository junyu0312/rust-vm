bitflags::bitflags! {
    #[derive(Clone, Copy)]
    pub struct PciCommand: u16 {
        const IO = 0x1;
        const MEMORY = 0x2;
        const MASTER = 0x4;
        const SPECIAL = 0x8;
        const INVALIDATE = 0x10;
        const VGA_PALETTE = 0x20;
        const PARITY = 0x40;
        const WAIT = 0x80;
        const SERR = 0x100;
        const FAST_BACK = 0x200;
        const INTX_DISABLE = 0x400;
    }
}
