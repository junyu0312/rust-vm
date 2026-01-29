use std::io;
use std::io::Write;

use strum_macros::FromRepr;
use vm_core::device::Device;
use vm_core::device::mmio::MmioDevice;
use vm_core::device::mmio::MmioRange;
use vm_core::irq::arch::aarch64::GIC_SPI;
use vm_core::irq::arch::aarch64::IRQ_TYPE_LEVEL_HIGH;
use vm_fdt::FdtWriter;

use crate::device::pl011::cr::Cr;
use crate::device::pl011::dr::Dr;
use crate::device::pl011::fbrd::Fbrd;
use crate::device::pl011::fr::Fr;
use crate::device::pl011::ibrd::Ibrd;
use crate::device::pl011::ifls::Ifls;
use crate::device::pl011::imsc::Imsc;
use crate::device::pl011::lcrh::LcrH;

mod dr {
    use bitflags::bitflags;

    bitflags! {
        #[derive(Default)]
        pub struct Dr: u16 {
            const DATA = 1 << 7 | 1 << 6 |
                1 << 5 | 1 << 4 | 1 << 3 |
                1 << 2 | 1 << 1 | 1 << 0;
            const FE = 1 << 8;
            const PE = 1 << 9;
            const BE = 1 << 10;
            const OE = 1 << 11;
        }
    }
}

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

#[allow(dead_code)]
pub struct Pl011 {
    mmio_range: MmioRange,

    dr: Dr,
    fr: Fr,
    ibrd: Ibrd,
    fbrd: Fbrd,
    lcr_h: LcrH,
    cr: Cr,
    ifls: Ifls,
    imsc: Imsc,

    tx_fifo: [u8; 32],
    tx_r_cursor: usize,
    tx_w_cursor: usize,
    rx_fifo: [u16; 32], // 12bit-wide(data and error bits)
    rx_r_cursor: usize,
    rx_w_cursor: usize,
}

impl Pl011 {
    pub fn new(mmio_range: MmioRange) -> Self {
        Pl011 {
            mmio_range,
            dr: Dr::default(),
            fr: Fr::default(),
            ibrd: Ibrd::default(),
            fbrd: Fbrd::default(),
            lcr_h: LcrH::default(),
            cr: Cr::default(),
            ifls: Ifls::default(),
            imsc: Imsc::default(),
            tx_fifo: Default::default(),
            tx_r_cursor: Default::default(),
            tx_w_cursor: Default::default(),
            rx_fifo: Default::default(),
            rx_r_cursor: Default::default(),
            rx_w_cursor: Default::default(),
        }
    }

    fn fifo_enabled(&self) -> bool {
        self.lcr_h.fen()
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

        if self.fifo_enabled() {
            self.tx_fifo[self.tx_w_cursor] = data[0];
            self.tx_w_cursor += 1;
        } else {
            self.tx_fifo[0] = data[0];
        }
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

    fn write_imsc(&mut self, len: usize, data: &[u8]) {
        let mut buf = [0; 2];
        buf[0..len].copy_from_slice(data);
        self.imsc = Imsc::from_bits_truncate(u16::from_le_bytes(buf));
    }

    fn write_icr(&mut self, _len: usize, _data: &[u8]) {
        // TODO: clear interrupt
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
}

impl Device for Pl011 {
    fn name(&self) -> &str {
        "pl011"
    }

    fn as_mmio_device(&self) -> Option<&dyn MmioDevice> {
        Some(self)
    }

    fn as_mmio_device_mut(&mut self) -> Option<&mut dyn MmioDevice> {
        Some(self)
    }
}

impl MmioDevice for Pl011 {
    fn mmio_ranges(&self) -> Vec<MmioRange> {
        vec![self.mmio_range]
    }

    fn mmio_read(&mut self, offset: u64, len: usize, data: &mut [u8]) {
        let offset: u16 = offset.try_into().unwrap();
        let reg = Register::from_repr(offset).unwrap();
        match reg {
            Register::Dr => todo!(),
            Register::Fr => self.read_fr(len, data),
            Register::Ibrd => self.read_ibrd(len, data),
            Register::Fbrd => self.read_fbrd(len, data),
            Register::LcrH => self.read_lcr_h(len, data),
            Register::Cr => self.read_cr(len, data),
            Register::Ifls => self.read_ifls(len, data),
            Register::Imsc => self.read_imsc(len, data),
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
        fdt.property_array_u32("interrupts", &[GIC_SPI, 1, IRQ_TYPE_LEVEL_HIGH])?;
        fdt.property_array_u32("clocks", &[3, 3])?;
        fdt.property_string_list(
            "clock-names",
            vec!["uartclk".to_string(), "apb_pclk".to_string()],
        )?;
        fdt.end_node(node)?;

        Ok(())
    }
}
