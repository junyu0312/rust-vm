use std::io;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use bitflags::Flags;
use strum_macros::FromRepr;
use vm_core::device::Device;
use vm_core::device::mmio::MmioRange;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_core::irq::InterruptController;
use vm_core::irq::arch::aarch64::GIC_SPI;
use vm_core::irq::arch::aarch64::IRQ_TYPE_LEVEL_HIGH;
use vm_fdt::FdtWriter;

use crate::device::pl011::cr::Cr;
use crate::device::pl011::fbrd::Fbrd;
use crate::device::pl011::fr::Fr;
use crate::device::pl011::ibrd::Ibrd;
use crate::device::pl011::ifls::Ifls;
use crate::device::pl011::imsc::Imsc;
use crate::device::pl011::lcrh::LcrH;
use crate::device::pl011::ris::Ris;

mod fr {
    use bitflags::bitflags;

    bitflags! {
        pub struct Fr: u16 {
            const CTS = 1 << 0;
            const DSR = 1 << 1;
            const DCD = 1 << 2;
            const BUSY = 1 << 3;
            const RXFE = 1 << 4;
            const TXFF = 1 << 5;
            const RXFF = 1 << 6;
            const TXFE = 1 << 7;
            const RI = 1 << 8;
        }
    }

    impl Default for Fr {
        fn default() -> Self {
            // The UARTFR Register is the flag register.
            // After reset TXFF, RXFF, and BUSY are 0, and TXFE and RXFE are 1.
            Fr::TXFE | Fr::RXFE
        }
    }
}

mod ibrd {
    #[derive(Default)]
    pub struct Ibrd(u16);

    impl Ibrd {
        pub fn write(&mut self, val: u16) {
            self.0 = val;
        }

        pub fn read(&self) -> u16 {
            self.0
        }
    }
}

mod fbrd {
    #[derive(Default)]
    pub struct Fbrd(u8);

    impl Fbrd {
        pub fn write(&mut self, val: u8) {
            self.0 = val;
        }

        pub fn read(&self) -> u8 {
            self.0
        }
    }
}

mod lcrh {
    const MASK: u16 = 0x7f;

    #[derive(Default)]
    pub struct LcrH(u16);

    impl LcrH {
        pub fn write(&mut self, val: u16) {
            self.0 = val & MASK;
        }

        pub fn read(&self) -> u16 {
            self.0
        }

        pub fn fen(&self) -> bool {
            self.0 & 0x8 != 0
        }
    }
}

mod cr {
    use bitflags::bitflags;

    bitflags! {
        pub struct Cr: u16 {
            const CTSEn = 1 << 15;
            const RTSEn = 1 << 14;
            const Out2 = 1 << 13;
            const Out1 = 1 << 12;
            const RTS = 1 << 11;
            const DTR = 1 << 10;
            const RXE = 1 << 9;
            const TXE = 1 << 8;
            const LBE = 1 << 7;
            const SIRLP = 1 << 2;
            const SIREN = 1 << 1;
            const UARTEN = 1 << 0;
        }
    }

    impl Default for Cr {
        fn default() -> Self {
            Cr::TXE | Cr::RXE
        }
    }
}

mod ifls {
    pub struct Ifls(u16);

    impl Default for Ifls {
        fn default() -> Self {
            Ifls(0x12)
        }
    }

    impl Ifls {
        pub fn write(&mut self, val: u16) {
            self.0 = val;
        }

        pub fn read(&self) -> u16 {
            self.0
        }
    }
}

mod imsc {
    use bitflags::bitflags;

    bitflags! {
        #[derive(Default)]
        pub struct Imsc: u16 {
            const OEIM = 1 << 10;
            const BEIM = 1 << 9;
            const PEIM = 1 << 8;
            const FEIM = 1 << 7;
            const RTIM = 1 << 6;
            const TXIM = 1 << 5;
            const RXIM = 1 << 4;
            const DSRMIM = 1 << 3;
            const DCDMIM = 1 << 2;
            const CTSMIM = 1 << 1;
            const RIMIM = 1 << 0;
        }
    }
}

mod ris {
    use bitflags::bitflags;

    bitflags! {
        #[derive(Default)]
        pub struct Ris: u16 {
            const OERIS = 1 << 10;
            const BERIS = 1 << 9;
            const PERIS = 1 << 8;
            const FERIS = 1 << 7;
            const RTRIS = 1 << 6;
            const TXRIS = 1 << 5;
            const RXRIS = 1 << 4;
            const DSRRMIS = 1 << 3;
            const DCDRMIS = 1 << 2;
            const CTSRMIS = 1 << 1;
            const RIRMIS = 1 << 0;
        }
    }
}

mod icr {
    use bitflags::bitflags;

    bitflags! {
        #[derive(Default)]
        pub struct Icr: u16 {
            const OEIC = 1 << 10;
            const BEIC = 1 << 9;
            const PEIC = 1 << 8;
            const FEIC = 1 << 7;
            const RTIC = 1 << 6;
            const TXIC = 1 << 5;
            const RXIC = 1 << 4;
            const DSRMIC = 1 << 3;
            const DCDMIC = 1 << 2;
            const CTSMIC = 1 << 1;
            const RIMIC = 1 << 0;
        }
    }
}

#[derive(Debug, FromRepr)]
#[repr(u16)]
enum Register {
    Dr = 0x00,
    Fr = 0x18,
    Ibrd = 0x24,
    Fbrd = 0x28,
    LcrH = 0x2c,
    Cr = 0x30,
    Ifls = 0x34,
    Imsc = 0x38,
    Ris = 0x3c,
    Mis = 0x40,
    Icr = 0x44,
    Macr = 0x48,
    PeriphID0 = 0xfe0,
    PeriphID1 = 0xfe4,
    PeriphID2 = 0xfe8,
    PeriphID3 = 0xfec,
    CellID0 = 0xff0,
    CellId1 = 0xff4,
    CellId2 = 0xff8,
    CellId3 = 0xffc,
}

struct Pl011Internal<const IRQ: u32> {
    mmio_range: MmioRange,
    irq_chip: Arc<dyn InterruptController>,

    fr: Fr,
    ibrd: Ibrd,
    fbrd: Fbrd,
    lcr_h: LcrH,
    cr: Cr,
    ifls: Ifls,
    imsc: Imsc,
    ris: Ris,

    rx_fifo: [Option<u16>; 32], // 12bit-wide(data and error bits), option to indicate thr's status
    rx_r_cursor: usize,
    rx_w_cursor: usize,
}

impl<const IRQ: u32> Pl011Internal<IRQ> {
    fn new(mmio_range: MmioRange, irq_chip: Arc<dyn InterruptController>) -> Self {
        Pl011Internal {
            mmio_range,
            irq_chip,
            fr: Fr::default(),
            ibrd: Ibrd::default(),
            fbrd: Fbrd::default(),
            lcr_h: LcrH::default(),
            cr: Cr::default(),
            ifls: Ifls::default(),
            imsc: Imsc::default(),
            ris: Ris::default(),
            rx_fifo: Default::default(),
            rx_r_cursor: Default::default(),
            rx_w_cursor: Default::default(),
        }
    }

    fn fifo_enabled(&self) -> bool {
        self.lcr_h.fen()
    }

    fn read_dr(&mut self, len: usize, data: &mut [u8]) {
        assert_eq!(len, 2);
        if self.fifo_enabled() {
            let d = self.rx_fifo[self.rx_r_cursor].unwrap_or_default();
            self.rx_r_cursor = (self.rx_r_cursor + 1) % 32;
            data.copy_from_slice(&d.to_le_bytes());
        } else {
            let d = self.rx_fifo[0].unwrap_or_default();
            self.rx_fifo[0] = None;
            data.copy_from_slice(&d.to_le_bytes());
        }

        self.update();
    }

    fn write_dr(&mut self, _len: usize, data: &[u8]) {
        /*
         For words to be transmitted:
         • if the FIFOs are enabled, data written to this location is
           pushed onto the transmit FIFO
         • if the FIFOs are not enabled, data is stored in the transmitter
           holding register (the bottom word of the transmit FIFO).
        */
        print!("{}", data[0] as char);
        io::stdout().flush().unwrap();

        // if self.fifo_enabled() {
        //     self.tx_fifo[self.tx_w_cursor] = data[0];
        //     self.tx_w_cursor += 1;
        // } else {
        //     self.tx_fifo[0] = data[0];
        // }
    }

    fn read_fr(&self, _len: usize, data: &mut [u8]) {
        data[0..2].copy_from_slice(&self.fr.bits().to_le_bytes());
    }

    fn read_ibrd(&self, _len: usize, data: &mut [u8]) {
        data[0..2].copy_from_slice(&self.ibrd.read().to_le_bytes());
    }

    fn write_ibrd(&mut self, len: usize, data: &[u8]) {
        assert_eq!(len, 2);
        assert_eq!(data.len(), 2);
        self.ibrd
            .write(u16::from_le_bytes(data.try_into().unwrap()));
    }

    fn read_fbrd(&self, _len: usize, data: &mut [u8]) {
        data[0] = self.fbrd.read();
    }

    fn write_fbrd(&mut self, _len: usize, data: &[u8]) {
        self.fbrd.write(data[0]);
    }

    fn read_lcr_h(&self, _len: usize, data: &mut [u8]) {
        data[0..2].copy_from_slice(&self.lcr_h.read().to_le_bytes());
    }

    fn write_lcr_h(&mut self, len: usize, data: &[u8]) {
        assert_eq!(len, 2);
        assert_eq!(data.len(), 2);
        self.lcr_h
            .write(u16::from_le_bytes(data.try_into().unwrap()));
    }

    fn read_cr(&self, len: usize, data: &mut [u8]) {
        assert_eq!(len, 2);
        assert_eq!(data.len(), 2);
        data.copy_from_slice(&self.cr.bits().to_le_bytes());
    }

    fn write_cr(&mut self, len: usize, data: &[u8]) {
        assert_eq!(len, 2);
        assert_eq!(data.len(), 2);
        self.cr = Cr::from_bits_truncate(u16::from_le_bytes(data.try_into().unwrap()));
    }

    fn read_ifls(&self, _len: usize, data: &mut [u8]) {
        data[0..2].copy_from_slice(&self.ifls.read().to_le_bytes());
    }

    fn write_ifls(&mut self, _len: usize, data: &[u8]) {
        self.ifls
            .write(u16::from_le_bytes(data.try_into().unwrap()));
    }

    fn read_imsc(&self, _len: usize, data: &mut [u8]) {
        data[0..2].copy_from_slice(&self.imsc.bits().to_le_bytes());
    }

    fn read_ris(&self, _len: usize, data: &mut [u8]) {
        data[0..2].copy_from_slice(&self.ris.bits().to_le_bytes());
    }

    fn write_imsc(&mut self, len: usize, data: &[u8]) {
        let mut buf = [0; 2];
        buf[0..len].copy_from_slice(data);
        self.imsc = Imsc::from_bits_truncate(u16::from_le_bytes(buf));
    }

    fn write_icr(&mut self, len: usize, data: &[u8]) {
        assert_eq!(len, 2);
        assert_eq!(data.len(), 2);

        let _icr = u16::from_le_bytes(data.try_into().unwrap());

        // I dont know why, the Linux kernel always write 0
        // self.ris.remove(Ris::from_bits_truncate(icr));
        self.ris.clear();
        self.update();
    }

    fn read_periph_id0(&self, data: &mut [u8]) {
        // data.fill(0);
        data[0] = 0x11;
    }

    fn read_periph_id1(&self, data: &mut [u8]) {
        // data.fill(0);
        data[0] = 0x10;
    }

    fn read_periph_id2(&self, data: &mut [u8]) {
        // data.fill(0);
        data[0] = 0x04;
    }

    fn read_periph_id3(&self, data: &mut [u8]) {
        // data.fill(0);
        data[0] = 0x00;
    }

    fn read_cell_id0(&self, data: &mut [u8]) {
        // data.fill(0);
        data[0] = 0x0d;
    }

    fn read_cell_id1(&self, data: &mut [u8]) {
        // data.fill(0);
        data[0] = 0xf0;
    }

    fn read_cell_id2(&self, data: &mut [u8]) {
        // data.fill(0);
        data[0] = 0x05;
    }

    fn read_cell_id3(&self, data: &mut [u8]) {
        // data.fill(0);
        data[0] = 0xb1;
    }

    fn stdio(&mut self, e: u8) {
        if self.fr.contains(Fr::RXFF) {
            return;
        }

        if self.fifo_enabled() {
            self.rx_fifo[self.rx_w_cursor] = Some(e as u16);
            self.rx_w_cursor = (self.rx_w_cursor + 1) % 32;
        } else {
            self.rx_fifo[0] = Some(e as u16);
        }

        self.update();
    }

    fn update(&mut self) {
        if self.fifo_enabled() {
            if (self.rx_w_cursor + 1) % 32 == self.rx_r_cursor {
                self.fr.insert(Fr::RXFF);
            } else {
                self.fr.remove(Fr::RXFF);
            }

            if self.rx_w_cursor == self.rx_r_cursor {
                self.fr.insert(Fr::RXFE);
            } else {
                self.fr.remove(Fr::RXFE);
            }
        } else {
            if self.rx_fifo[0].is_some() {
                self.fr.insert(Fr::RXFF);
            } else {
                self.fr.remove(Fr::RXFF);
            }

            if self.rx_fifo[0].is_none() {
                self.fr.insert(Fr::RXFE);
            } else {
                self.fr.remove(Fr::RXFE);
            }
        }

        if !self.fr.contains(Fr::RXFE) {
            self.ris.insert(Ris::RXRIS);
        }

        if self.imsc.contains(Imsc::RXIM) && self.ris.contains(Ris::RXRIS) {
            self.trigger_irq(true);
        } else {
            self.trigger_irq(false);
        }
    }

    fn trigger_irq(&self, active: bool) {
        self.irq_chip.trigger_irq(32 + IRQ, active);
    }
}

impl<const IRQ: u32> Device for Pl011Internal<IRQ> {
    fn name(&self) -> String {
        "pl011".to_string()
    }
}

impl<const IRQ: u32> MmioDevice for Pl011Internal<IRQ> {
    fn mmio_range(&self) -> MmioRange {
        self.mmio_range
    }

    fn mmio_read(&mut self, offset: u64, len: usize, data: &mut [u8]) {
        let offset: u16 = offset.try_into().unwrap();
        let reg = Register::from_repr(offset).unwrap();

        match reg {
            Register::Dr => self.read_dr(len, data),
            Register::Fr => self.read_fr(len, data),
            Register::Ibrd => self.read_ibrd(len, data),
            Register::Fbrd => self.read_fbrd(len, data),
            Register::LcrH => self.read_lcr_h(len, data),
            Register::Cr => self.read_cr(len, data),
            Register::Ifls => self.read_ifls(len, data),
            Register::Imsc => self.read_imsc(len, data),
            Register::Ris => self.read_ris(len, data),
            Register::Mis => todo!(),
            Register::Icr => unreachable!("WO"),
            Register::Macr => todo!(),
            Register::PeriphID0 => self.read_periph_id0(data),
            Register::PeriphID1 => self.read_periph_id1(data),
            Register::PeriphID2 => self.read_periph_id2(data),
            Register::PeriphID3 => self.read_periph_id3(data),
            Register::CellID0 => self.read_cell_id0(data),
            Register::CellId1 => self.read_cell_id1(data),
            Register::CellId2 => self.read_cell_id2(data),
            Register::CellId3 => self.read_cell_id3(data),
        }
    }

    fn mmio_write(&mut self, offset: u64, len: usize, data: &[u8]) {
        let offset: u16 = offset.try_into().unwrap();

        let reg = Register::from_repr(offset).unwrap();
        match reg {
            Register::Dr => self.write_dr(len, data),
            Register::Fr => unreachable!("RO"),
            Register::Ibrd => self.write_ibrd(len, data),
            Register::Fbrd => self.write_fbrd(len, data),
            Register::LcrH => self.write_lcr_h(len, data),
            Register::Cr => self.write_cr(len, data),
            Register::Ifls => self.write_ifls(len, data),
            Register::Imsc => self.write_imsc(len, data),
            Register::Ris => (), // A write has no effect
            Register::Mis => (), // A write has no effect
            Register::Icr => self.write_icr(len, data),
            Register::Macr => todo!(),
            Register::PeriphID0 => unreachable!("RO"),
            Register::PeriphID1 => unreachable!("RO"),
            Register::PeriphID2 => unreachable!("RO"),
            Register::PeriphID3 => unreachable!("RO"),
            Register::CellID0 => unreachable!("RO"),
            Register::CellId1 => unreachable!("RO"),
            Register::CellId2 => unreachable!("RO"),
            Register::CellId3 => unreachable!("RO"),
        }
    }

    fn generate_dt(&self, fdt: &mut FdtWriter) -> Result<(), vm_fdt::Error> {
        let node = fdt.begin_node("uartclk")?;
        fdt.property_string("compatible", "fixed-clock")?;
        fdt.property_u32("#clock-cells", 0)?;
        fdt.property_u32("clock-frequency", 24000000)?;
        fdt.property_phandle(3)?;
        fdt.end_node(node)?;

        // let node = fdt.begin_node("apb_pclk")?;
        // fdt.property_string("compatible", "fixed-clock")?;
        // fdt.property_u32("#clock-cells", 0)?;
        // fdt.property_u32("clock-frequency", 24_000_000)?;
        // fdt.property_phandle(4)?; // phandle 4
        // fdt.end_node(node)?;

        let node = fdt.begin_node(&format!("serial@{:x}", self.mmio_range.start))?;
        fdt.property_string_list(
            "compatible",
            vec!["arm,pl011".to_string(), "arm,primecell".to_string()],
        )?;
        fdt.property_array_u64("reg", &[self.mmio_range.start, self.mmio_range.len as u64])?;
        fdt.property_array_u32("interrupts", &[GIC_SPI, IRQ, IRQ_TYPE_LEVEL_HIGH])?;
        fdt.property_array_u32("clocks", &[3, 3])?;
        fdt.property_string_list(
            "clock-names",
            vec!["uartclk".to_string(), "apb_pclk".to_string()],
        )?;
        fdt.end_node(node)?;

        Ok(())
    }
}

pub struct Pl011<const IRQ: u32>(Arc<Mutex<Pl011Internal<IRQ>>>);

impl<const IRQ: u32> Pl011<IRQ> {
    pub fn new(mmio_range: MmioRange, irq_chip: Arc<dyn InterruptController>) -> Self {
        let pl011 = Arc::new(Mutex::new(Pl011Internal::new(mmio_range, irq_chip)));

        thread::spawn({
            let pl011 = pl011.clone();
            move || {
                let stdin = io::stdin();
                let mut handle = stdin.lock();
                let mut buffer = [0u8; 1];

                while let Ok(n) = handle.read(&mut buffer) {
                    if n == 0 {
                        break;
                    }
                    let mut pl011 = pl011.lock().unwrap();
                    pl011.stdio(buffer[0]);
                }
            }
        });

        Self(pl011)
    }
}

impl<const IRQ: u32> Device for Pl011<IRQ> {
    fn name(&self) -> String {
        self.0.lock().unwrap().name()
    }
}

impl<const IRQ: u32> MmioDevice for Pl011<IRQ> {
    fn mmio_range(&self) -> MmioRange {
        self.0.lock().unwrap().mmio_range()
    }

    fn mmio_read(&mut self, offset: u64, len: usize, data: &mut [u8]) {
        self.0.lock().unwrap().mmio_read(offset, len, data);
    }

    fn mmio_write(&mut self, offset: u64, len: usize, data: &[u8]) {
        self.0.lock().unwrap().mmio_write(offset, len, data);
    }

    fn generate_dt(&self, fdt: &mut FdtWriter) -> Result<(), vm_fdt::Error> {
        self.0.lock().unwrap().generate_dt(fdt)
    }
}
