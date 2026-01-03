use std::io::Write;
use std::io::{self};

use crate::device::pio::PioDevice;

const DLAB: u8 = 0x80;
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
const DLL: u16 = PORT;
const DLH: u16 = PORT + 1;

#[derive(Default)]
pub struct Uart16550 {
    dll: u8,
    dlh: u8,
    ier: u8,
    fcr: u8,
    lcr: u8,
    mcr: u8,
}

impl Uart16550 {
    // Transmit register
    fn out_txr(&mut self, data: &[u8]) {
        assert!(data.len() == 1);
        let byte = data[0];

        print!("{}", byte as char);
        io::stdout().flush().unwrap();
    }

    // Receive register
    fn in_rxr(&self, _data: &mut [u8]) {
        // todo!()
        // TODO
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
    fn in_iir(&self, _data: &mut [u8]) {
        // todo!()
        // TODO
    }

    // FIFO Control
    fn out_fcr(&mut self, data: &[u8]) {
        assert!(data.len() == 1);
        self.fcr = data[0];
    }

    // Line Control
    fn out_lcr(&mut self, data: &[u8]) {
        assert!(data.len() == 1);
        self.lcr = data[0];
    }

    // Line Control
    fn in_lcr(&self, data: &mut [u8]) {
        data[0] = self.lcr;
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
}

impl PioDevice for Uart16550 {
    fn ports(&self) -> &[u16] {
        &[TXR, RXR, IER, IIR, FCR, LCR, MCR, LSR, MSR]
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
            _ => todo!(),
        }
    }

    fn io_out(&mut self, port: u16, data: &[u8]) {
        match port {
            TXR if self.lcr & DLAB == 0 => self.out_txr(data),
            DLL if self.lcr & DLAB != 0 => self.out_dll(data),
            IER if self.lcr & DLAB == 0 => self.out_ier(data),
            DLH if self.lcr & DLAB != 0 => self.out_dlh(data),
            FCR => self.out_fcr(data),
            LCR => self.out_lcr(data),
            MCR => self.out_mcr(data),
            _ => panic!("unsupported port {:#x} for uart16550", port),
        }
    }
}
