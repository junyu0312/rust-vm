use strum_macros::FromRepr;
use vm_core::device::Device;
use vm_core::device::mmio::MmioDevice;
use vm_core::device::mmio::MmioRange;
use vm_core::irq::arch::aarch64::GIC_SPI;
use vm_core::irq::arch::aarch64::IRQ_TYPE_LEVEL_HIGH;
use vm_fdt::FdtWriter;

use crate::device::pl011::dr::Dr;
use crate::device::pl011::fr::Fr;
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

mod lcrh {
    #[derive(Default)]
    pub struct LcrH(u32);

    impl LcrH {
        pub fn fen(&self) -> bool {
            self.0 & 0x8 != 0
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
    Msc = 0x38,
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

pub struct Pl011 {
    mmio_range: MmioRange,

    dr: Dr,
    fr: Fr,
    lcr_h: LcrH,

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
            lcr_h: LcrH::default(),
            tx_fifo: Default::default(),
            tx_r_cursor: Default::default(),
            tx_w_cursor: Default::default(),
            rx_fifo: Default::default(),
            rx_r_cursor: Default::default(),
            rx_w_cursor: Default::default(),
        }
    }
}
impl Pl011 {
    fn fifo_enabled(&self) -> bool {
        self.lcr_h.fen()
    }

    fn update_state(&mut self) {}

    fn write_dr(&mut self, len: usize, data: &[u8]) {
        assert!(len == 1 && data.len() == 1);

        /*
         For words to be transmitted:
         • if the FIFOs are enabled, data written to this location is
           pushed onto the transmit FIFO
         • if the FIFOs are not enabled, data is stored in the transmitter
           holding register (the bottom word of the transmit FIFO).
        */
        print!("{}", data[0] as char);

        if self.fifo_enabled() {
            self.tx_fifo[self.tx_w_cursor] = data[0];
            self.tx_w_cursor += 1;
        } else {
            self.tx_fifo[0] = data[0];
        }
    }

    fn read_fr(&self, len: usize, data: &mut [u8]) {
        assert_eq!(len, 4);
        assert_eq!(data.len(), 4);
        data[0..2].copy_from_slice(&self.fr.bits().to_le_bytes());
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
            Register::Ibrd => todo!(),
            Register::Fbrd => todo!(),
            Register::LcrH => todo!(),
            Register::Cr => todo!(),
            Register::Msc => todo!(),
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
            Register::Fr => unreachable!(),
            Register::Ibrd => todo!(),
            Register::Fbrd => todo!(),
            Register::LcrH => todo!(),
            Register::Cr => todo!(),
            Register::Msc => todo!(),
            Register::Macr => todo!(),
            Register::PeriphID0 => unreachable!(),
            Register::PeriphID1 => unreachable!(),
            Register::PeriphID2 => unreachable!(),
            Register::PeriphID3 => unreachable!(),
            Register::CellID0 => unreachable!(),
            Register::CellId1 => unreachable!(),
            Register::CellId2 => unreachable!(),
            Register::CellId3 => unreachable!(),
        }
    }

    fn generate_dt(&self, fdt: &mut FdtWriter) -> Result<(), vm_fdt::Error> {
        /*
        uart0: serial@12100000 {
            compatible = "arm,pl011", "arm,primecell";
            reg = <0x12100000 0x1000>;
            interrupts = <GIC_SPI 4 IRQ_TYPE_LEVEL_HIGH>;
            clocks = <&crg HI3519_UART0_CLK>, <&crg HI3519_UART0_CLK>;
            clock-names = "uartclk", "apb_pclk";
            status = "disabled";
        };
         */

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
        println!("{:?}", self.mmio_range);
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
