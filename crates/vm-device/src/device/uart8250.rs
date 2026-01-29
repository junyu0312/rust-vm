use std::io::Write;
use std::io::{self};
use std::sync::Arc;

use vm_core::device::Device;
use vm_core::device::mmio::MmioDevice;
use vm_core::device::mmio::MmioRange;
use vm_core::device::pio::PioDevice;
use vm_core::device::pio::PortRange;
use vm_core::irq::InterruptController;
use vm_fdt::FdtWriter;

use crate::device::uart8250::ier::IER;
use crate::device::uart8250::iir::IIR;
use crate::device::uart8250::lcr::LCR;
use crate::device::uart8250::lsr::LSR;
use crate::device::uart8250::mcr::MCR;
use crate::device::uart8250::msr::MSR;

/*
 * https://www.lammertbies.nl/comm/info/serial-uart
 * https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming
 */

const THR: u16 = 0;
const RBR: u16 = 0;
const IER: u16 = 1;
const IIR: u16 = 2;
const FCR: u16 = 2;
const LCR: u16 = 3;
const MCR: u16 = 4;
const LSR: u16 = 5;
const MSR: u16 = 6;
const SR: u16 = 7;
const DLL: u16 = 0;
const DLH: u16 = 1;

mod ier {
    use bitflags::bitflags;

    bitflags! {
        pub struct IER: u8 {
            const ReceiveDataAvailable = 1 << 0;
            const TransmitterHoldingRegisterEmpty = 1 << 1;
            const ReceiverLineStatusRegisterChange = 1 << 2;
            const ModemStatusRegisterChange = 1 << 3;
        }
    }

    impl Default for IER {
        fn default() -> Self {
            let mut ier = IER::empty();
            ier.insert(IER::ReceiveDataAvailable | IER::TransmitterHoldingRegisterEmpty);
            ier
        }
    }
}
mod iir {
    use bitflags::bitflags;

    bitflags! {
        #[derive(Clone, Copy)]
        pub struct IIR: u8 {
            const InterruptPending = 1 << 0;
            const IterruptIdBit1 = 1 << 1;
            const IterruptIdBit2 = 1 << 2;
            const IterruptIdBit3 = 1 << 3;
        }
    }

    impl Default for IIR {
        fn default() -> Self {
            let mut iir = IIR::empty();
            // 1 means no interrupt pending
            iir.insert(IIR::InterruptPending);
            iir
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

mod mcr {
    use bitflags::bitflags;

    bitflags! {
        pub struct MCR: u8 {
            const DTR = 1 << 0;
            const RequestToSend = 1 << 1;
            const AuxOut1 = 1 << 2;
            const AuxOut2 = 1 << 3;
            const LoopbackMode = 1 << 4;
        }
    }

    impl Default for MCR {
        fn default() -> Self {
            let mut mcr = MCR::empty();
            mcr.insert(MCR::AuxOut2);
            mcr
        }
    }
}

mod lsr {
    use bitflags::bitflags;

    bitflags! {
        pub struct LSR: u8 {
            const DataReady = 1 << 0;
            const OverrunError = 1 << 1;
            const ParityError = 1 << 2;
            const FramingError = 1 << 3;
            const BreakSignalReceived = 1 << 4;
            const THREmpty = 1 << 5;
            const THREmptyAndLineIdle = 1 << 6;
        }
    }

    impl Default for LSR {
        fn default() -> Self {
            let mut lsr = LSR::empty();
            lsr.insert(LSR::THREmpty | LSR::THREmptyAndLineIdle);
            lsr
        }
    }
}

mod msr {
    use bitflags::bitflags;

    bitflags! {
        #[derive(Default)]
        pub struct MSR: u8 {
            const DeltaClearToSend = 1 << 0;
            const DeltaDataSetReady = 1 << 1;
            const TrailingEdgeRingIndicator = 1 << 2;
            const DeltaDataCarrierDetect = 1 << 3;
            const ClearToSend = 1 << 4;
            const DataSetReady = 1 << 5;
            const RingIndicator = 1 << 6;
            const CarrierDetect = 1 << 7;
        }
    }
}

pub struct Uart8250<const IRQ: u32> {
    port_base: Option<u16>,
    mmio_range: Option<MmioRange>,

    txr: u8,
    rbr: Option<u8>,
    dll: u8,
    dlh: u8,
    ier: IER,
    iir: IIR,
    // fcr: u8,
    lcr: LCR,
    mcr: MCR,
    lsr: LSR,
    msr: MSR,
    irq_controller: Arc<dyn InterruptController>,
    irq_state: bool,
}

impl<const IRQ: u32> Uart8250<IRQ> {
    pub fn new(
        port_base: Option<u16>,
        mmio_range: Option<MmioRange>,
        irq_controller: Arc<dyn InterruptController>,
    ) -> Self {
        Uart8250 {
            port_base,
            mmio_range,
            txr: Default::default(),
            rbr: Default::default(),
            dll: Default::default(),
            dlh: Default::default(),
            ier: Default::default(),
            iir: Default::default(),
            lcr: Default::default(),
            mcr: Default::default(),
            lsr: Default::default(),
            msr: Default::default(),
            irq_controller,
            irq_state: false,
        }
    }
}

impl<const IRQ: u32> Uart8250<IRQ> {
    // Transmitter holding register
    fn out_thr(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 1);
        let data = data[0];

        self.txr = data;

        // TODO: Extract
        {
            print!("{}", data as char);
            io::stdout().flush().unwrap();
            self.lsr.insert(LSR::THREmpty | LSR::THREmptyAndLineIdle);
        }
    }

    // fn receive_byte(&mut self, data: u8) {
    //     if self.rbr.is_some() {
    //         self.lsr.insert(LSR::OverrunError);
    //     } else {
    //         self.rbr = Some(data);
    //         self.lsr.insert(LSR::DataReady);
    //     }
    // }

    // Receive buffer register
    fn in_rbr(&mut self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);

        data[0] = self.rbr.take().unwrap_or_default();
        self.lsr.remove(LSR::DataReady);
    }

    // Divisor Latch Low
    fn out_dll(&mut self, data: &[u8]) {
        assert!(data.len() == 1);
        self.dll = data[0];
    }

    // Divisor Latch Low
    fn in_dll(&self, data: &mut [u8]) {
        assert!(data.len() == 1);
        data[0] = self.dll;
    }

    // Interrupt Enable
    fn out_ier(&mut self, data: &[u8]) {
        assert!(data.len() == 1);
        self.ier = IER::from_bits_truncate(data[0]);
    }

    // Interrupt Enable
    fn in_ier(&self, data: &mut [u8]) {
        assert!(data.len() == 1);
        data[0] = self.ier.bits();
    }

    // Divisor latch High
    fn out_dlh(&mut self, data: &[u8]) {
        assert!(data.len() == 1);
        self.dlh = data[0];
    }

    // Divisor latch High
    fn in_dlh(&mut self, data: &mut [u8]) {
        assert!(data.len() == 1);
        data[0] = self.dlh;
    }

    // Interrupt ID
    fn in_iir(&mut self, data: &mut [u8]) {
        assert!(data.len() == 1);
        data[0] = self.iir.bits();
    }

    // Line Control
    fn out_lcr(&mut self, data: &[u8]) {
        assert!(data.len() == 1);
        self.lcr = LCR::from_bits_truncate(data[0]);
    }

    // Line Control
    fn in_lcr(&self, data: &mut [u8]) {
        assert!(data.len() == 1);
        data[0] = self.lcr.bits();
    }

    // Modem Control
    fn out_mcr(&mut self, data: &[u8]) {
        assert!(data.len() == 1);
        self.mcr = MCR::from_bits_truncate(data[0]);
    }

    // Modem Control
    fn in_mcr(&self, data: &mut [u8]) {
        assert!(data.len() == 1);
        data[0] = self.mcr.bits();
    }

    // Line Status
    fn in_lsr(&mut self, data: &mut [u8]) {
        assert!(data.len() == 1);
        data[0] = self.lsr.bits();
    }

    // Modem Status
    fn in_msr(&self, data: &mut [u8]) {
        assert!(data.len() == 1);
        data[0] = self.msr.bits();
    }

    // Scratch Register
    fn in_sr(&self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        // Always return `0x00` to emualte 8250
        data[0] = 0x00;
    }

    fn update_irq(&mut self) {
        let mut iir = IIR::empty();

        if self.ier.contains(IER::ReceiveDataAvailable) && self.lsr.contains(LSR::DataReady) {
            iir.insert(IIR::IterruptIdBit2);
        }

        if self.ier.contains(IER::TransmitterHoldingRegisterEmpty)
            && self.lsr.contains(LSR::THREmpty)
        {
            iir.insert(IIR::IterruptIdBit1);
        }

        let irq_state = !iir.is_empty();
        if iir.is_empty() {
            iir.insert(IIR::InterruptPending);
        }

        self.iir = iir;
        if self.irq_state != irq_state {
            self.irq_state = irq_state;
            self.irq_controller.trigger_irq(IRQ, irq_state);
        }

        // if !self.ier.contains(IER::TransmitterHoldingRegisterEmpty) {
        self.lsr.insert(LSR::THREmpty | LSR::THREmptyAndLineIdle);
        // }
    }
}

impl<const IRQ: u32> Device for Uart8250<IRQ> {
    fn name(&self) -> &str {
        "uart8250"
    }

    fn as_pio_device(&self) -> Option<&dyn PioDevice> {
        if self.port_base.is_some() {
            Some(self)
        } else {
            None
        }
    }

    fn as_pio_device_mut(&mut self) -> Option<&mut dyn PioDevice> {
        if self.port_base.is_some() {
            Some(self)
        } else {
            None
        }
    }

    fn as_mmio_device(&self) -> Option<&dyn MmioDevice> {
        if self.mmio_range.is_some() {
            Some(self)
        } else {
            None
        }
    }

    fn as_mmio_device_mut(&mut self) -> Option<&mut dyn MmioDevice> {
        if self.mmio_range.is_some() {
            Some(self)
        } else {
            None
        }
    }
}

impl<const IRQ: u32> PioDevice for Uart8250<IRQ> {
    fn ports(&self) -> Vec<PortRange> {
        let base = *self.port_base.as_ref().unwrap();

        vec![
            PortRange {
                start: base,
                len: 1,
            },
            PortRange {
                start: base + 1,
                len: 1,
            },
            PortRange {
                start: base + 2,
                len: 1,
            },
            PortRange {
                start: base + 3,
                len: 1,
            },
            PortRange {
                start: base + 4,
                len: 1,
            },
            PortRange {
                start: base + 5,
                len: 1,
            },
            PortRange {
                start: base + 6,
                len: 1,
            },
            PortRange {
                start: base + 7,
                len: 1,
            },
        ]
    }

    fn io_in(&mut self, port: u16, data: &mut [u8]) {
        let base = self.port_base.as_ref().unwrap();

        match port - base {
            RBR if !self.lcr.is_dlab_set() => self.in_rbr(data),
            DLL if self.lcr.is_dlab_set() => self.in_dll(data),
            IER if !self.lcr.is_dlab_set() => self.in_ier(data),
            DLH if self.lcr.is_dlab_set() => self.in_dlh(data),
            IIR => self.in_iir(data),
            LCR => self.in_lcr(data),
            MCR => self.in_mcr(data),
            LSR => self.in_lsr(data),
            MSR => self.in_msr(data),
            SR => self.in_sr(data),
            _ => unreachable!(),
        }

        self.update_irq();
    }

    fn io_out(&mut self, port: u16, data: &[u8]) {
        let base = self.port_base.as_ref().unwrap();

        match port - base {
            THR if !self.lcr.is_dlab_set() => self.out_thr(data),
            DLL if self.lcr.is_dlab_set() => self.out_dll(data),
            IER if !self.lcr.is_dlab_set() => self.out_ier(data),
            DLH if self.lcr.is_dlab_set() => self.out_dlh(data),
            // FCR => self.out_fcr(data),
            FCR => (), // no fifo,
            LCR => self.out_lcr(data),
            MCR => self.out_mcr(data),
            LSR => (), // Ignore
            MSR => (), // Ignore
            SR => (),  // 8250 does not have sr
            _ => unreachable!(),
        }

        self.update_irq();
    }
}

impl<const IRQ: u32> MmioDevice for Uart8250<IRQ> {
    fn mmio_ranges(&self) -> Vec<MmioRange> {
        if let Some(mmio_range) = self.mmio_range {
            vec![mmio_range]
        } else {
            vec![]
        }
    }

    fn mmio_read(&mut self, offset: u64, _len: usize, data: &mut [u8]) {
        match offset as u16 {
            RBR if !self.lcr.is_dlab_set() => self.in_rbr(data),
            DLL if self.lcr.is_dlab_set() => self.in_dll(data),
            IER if !self.lcr.is_dlab_set() => self.in_ier(data),
            DLH if self.lcr.is_dlab_set() => self.in_dlh(data),
            IIR => self.in_iir(data),
            LCR => self.in_lcr(data),
            MCR => self.in_mcr(data),
            LSR => self.in_lsr(data),
            MSR => self.in_msr(data),
            SR => self.in_sr(data),
            _ => unreachable!(),
        }

        self.update_irq();
    }

    fn mmio_write(&mut self, offset: u64, _len: usize, data: &[u8]) {
        match offset as u16 {
            THR if !self.lcr.is_dlab_set() => self.out_thr(data),
            DLL if self.lcr.is_dlab_set() => self.out_dll(data),
            IER if !self.lcr.is_dlab_set() => self.out_ier(data),
            DLH if self.lcr.is_dlab_set() => self.out_dlh(data),
            // FCR => self.out_fcr(data),
            FCR => (), // no fifo,
            LCR => self.out_lcr(data),
            MCR => self.out_mcr(data),
            LSR => (), // Ignore
            MSR => (), // Ignore
            SR => (),  // 8250 does not have sr
            _ => unimplemented!("{offset}"),
        }

        self.update_irq();
    }

    fn generate_dt(&self, fdt: &mut FdtWriter) -> Result<(), vm_fdt::Error> {
        let Some(mmio_range) = self.mmio_range else {
            return Ok(());
        };

        let serial_node = fdt.begin_node(&format!("uart@{:x}", mmio_range.start))?;
        fdt.property_string("compatible", "ns16550a")?;
        fdt.property_array_u64("reg", &[mmio_range.start, mmio_range.len as u64])?;
        fdt.property_u32("clock-frequency", 24000000)?;
        fdt.property_u32("current-speed", 115200)?;
        fdt.property_array_u32("interrupts", &[0, 33, 4])?;
        fdt.property_phandle(2)?;
        fdt.end_node(serial_node)?;

        Ok(())
    }
}
