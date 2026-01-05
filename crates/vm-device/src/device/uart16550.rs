use std::io::Write;
use std::io::{self};

use crate::device::pio::PioDevice;
use crate::device::uart16550::iir::IIR;
use crate::device::uart16550::lcr::LCR;

const XMTRDY: u8 = 0x20;

const PORT: u16 = 0x3f8;
const TXR: u16 = PORT;
const RXR: u16 = PORT;
const IER: u16 = PORT + 1;
const IIR: u16 = PORT + 2;
const FCR: u16 = PORT + 2;
const LCR: u16 = PORT + 3;
const MCR: u16 = PORT + 4;
const LSR: u16 = PORT + 5;
const MSR: u16 = PORT + 6;
const SR: u16 = PORT + 7;
const DLL: u16 = PORT;
const DLH: u16 = PORT + 1;

mod iir {
    use bitflags::bitflags;

    bitflags! {
        #[derive(Default)]
        pub struct IIR:u8 {
            const InterruptPending = 1 << 0;
            const IterruptIdBit0 = 1 << 1;
            const IterruptIdBit1 = 1 << 2;
            const IterruptIdBit2 = 1 << 3;
            const Reserved0 = 1 << 4;
            const Reserved1 = 1 << 5;
            const FIFOsEnabled0 = 1 << 6;
            const FIFOsEnabled1 = 1 << 7;
        }

    }
}

mod lcr {
    use bitflags::bitflags;

    bitflags! {
        #[derive(Default)]
        pub struct LCR: u8 {
            const WLSB0 = 1 << 0;
            const WLSB1 = 1 << 1;
            const STB = 1 << 2;
            const PEN = 1 << 3;
            const EPS = 1 << 4;
            const StickParity = 1 << 5;
            const SetBreak = 1 << 6;
            const DLAB = 1 << 7;
        }
    }

    impl LCR {
        pub fn is_dlab_set(&self) -> bool {
            self.contains(LCR::DLAB)
        }
    }
}

#[derive(Default)]
pub struct Uart16550 {
    rxr: u8,
    dll: u8,
    dlh: u8,
    ier: u8,
    iir: IIR,
    fcr: u8,
    lcr: LCR,
    mcr: u8,
    sr: u8,
}

impl Uart16550 {
    // Transmit register
    fn out_txr(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 1);
        let byte = data[0];

        print!("{}", byte as char);
        io::stdout().flush().unwrap();
    }

    // Receive register
    fn in_rxr(&self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        data[0] = self.rxr;
    }

    // Divisor Latch Low
    fn out_dll(&mut self, data: &[u8]) {
        assert!(data.len() == 1);
        self.dll = data[0];
    }

    // Divisor latch High
    fn out_dlh(&mut self, data: &[u8]) {
        assert!(data.len() == 1);
        self.dlh = data[0];
    }

    // Interrupt Enable
    fn out_ier(&mut self, data: &[u8]) {
        assert!(data.len() == 1);
        self.ier = data[0];
    }

    fn in_ier(&self, data: &mut [u8]) {
        data[0] = self.ier;
    }

    // Interrupt ID
    fn in_iir(&self, data: &mut [u8]) {
        assert!(data.len() == 1);
        data[0] = self.iir.bits();
    }

    // FIFO Control
    fn out_fcr(&mut self, data: &[u8]) {
        assert!(data.len() == 1);
        self.fcr = data[0];
    }

    // Line Control
    fn out_lcr(&mut self, data: &[u8]) {
        assert!(data.len() == 1);
        self.lcr = LCR::from_bits_truncate(data[0]);
    }

    // Line Control
    fn in_lcr(&self, data: &mut [u8]) {
        data[0] = self.lcr.bits();
    }

    // Modem Control
    fn out_mcr(&mut self, data: &[u8]) {
        assert!(data.len() == 1);
        self.mcr = data[0];
    }

    // Modem Control
    fn in_mcr(&self, data: &mut [u8]) {
        data[0] = self.mcr;
    }

    // Line Status
    fn in_lsr(&self, data: &mut [u8]) {
        data[0] = XMTRDY;
    }

    // Modem Status
    fn in_msr(&self, _data: &mut [u8]) {
        todo!()
    }

    // Scratch Register
    fn out_sr(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 1);
        self.sr = data[0];
    }

    // Scratch Register
    fn in_sr(&self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        data[0] = self.sr;
    }
}

impl PioDevice for Uart16550 {
    fn ports(&self) -> &[u16] {
        &[TXR, RXR, IER, IIR, FCR, LCR, MCR, LSR, MSR, SR]
    }

    fn io_in(&mut self, port: u16, data: &mut [u8]) {
        match port {
            RXR => self.in_rxr(data),
            IER => self.in_ier(data),
            IIR => self.in_iir(data),
            LCR => self.in_lcr(data),
            MCR => self.in_mcr(data),
            LSR => self.in_lsr(data),
            MSR => self.in_msr(data),
            SR => self.in_sr(data),
            _ => todo!(),
        }
    }

    fn io_out(&mut self, port: u16, data: &[u8]) {
        match port {
            TXR if !self.lcr.is_dlab_set() => self.out_txr(data),
            DLL if self.lcr.is_dlab_set() => self.out_dll(data),
            IER if !self.lcr.is_dlab_set() => self.out_ier(data),
            DLH if self.lcr.is_dlab_set() => self.out_dlh(data),
            FCR => self.out_fcr(data),
            LCR => self.out_lcr(data),
            MCR => self.out_mcr(data),
            SR => self.out_sr(data),
            _ => panic!("unsupported port {:#x} for uart16550", port),
        }
    }
}
