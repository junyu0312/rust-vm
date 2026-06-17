use std::io::Write;
use std::io::{self};
use std::ops::Range;
use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use tracing::warn;
use vm_core::arch::irq::InterruptController;
use vm_core::device::Device;
use vm_core::device::error::DeviceError;
use vm_core::device::pio::pio_device::PioDevice;
use vm_utils::range_allocator::RangeAllocator;
use vm_utils::ring::Ring;

use crate::device::uart8250::ier::IER;
use crate::device::uart8250::lcr::LCR;
use crate::device::uart8250::lsr::LSR;
use crate::device::uart8250::mcr::MCR;
use crate::device::uart8250::msr::MSR;

/*
 * https://www.lammertbies.nl/comm/info/serial-uart
 * https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming
 */

const BUFFER_SIZE: usize = 16;

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

const FCR_CLEAR_RECEIVE_FIFO: u8 = 1 << 1;
const FCR_CLEAR_TRANSMIT_FIFO: u8 = 1 << 2;

const IIR_INTERRUPT_PENDING_FLAG: u8 = 1 << 0;
const IIR_TRANSMITTTER_HOLDING_REGISTER_EMPTY: u8 = 1 << 1;
const IIR_RECEIVER_DATA_AVAILABLE: u8 = 1 << 2;

mod ier {
    use bitflags::bitflags;

    bitflags! {
        #[derive(Default)]
        pub struct IER: u8 {
            const ReceiveDataAvailable = 1 << 0;
            const TransmitterHoldingRegisterEmpty = 1 << 1;
            const ReceiverLineStatusRegisterChange = 1 << 2;
            const ModemStatusRegisterChange = 1 << 3;
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
            const ErrornousDataInFifo = 1 << 7;
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

struct Uart8250Internal<const IRQ: u32> {
    irq_controller: Arc<dyn InterruptController>,

    // Transmitter holding register
    txr: Ring<BUFFER_SIZE, u8>,
    rbr: Ring<BUFFER_SIZE, u8>,
    dll: u8,
    dlh: u8,
    // Interrupt enable register
    ier: IER,
    // Interrupt identification register
    iir: u8,
    // FIFO Control Register
    fcr: u8,
    // Line Control Register
    lcr: LCR,
    // Modem Control Register
    mcr: MCR,
    // Line Status Register
    lsr: LSR,
    // Modem Status Register
    msr: MSR,
    // Scratch Register
    sr: u8,
    irq_state: bool,
}

impl<const IRQ: u32> Uart8250Internal<IRQ> {
    fn out_thr(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 1);
        let data = data[0];

        if self.txr.is_full() {
            warn!("uart txr is full, data {:?} is discarded", data);
            return;
        }
        self.txr.push(data).unwrap();

        // We reserved the push and pop to keep the semantics of uart,
        // I don't know if we need thr in the future.
        while let Some(c) = self.txr.try_pop() {
            print!("{}", c as char);
            io::stdout().flush().unwrap();
        }
    }

    // Receive buffer register
    fn in_rbr(&mut self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);

        if self.rbr.is_empty() {
            return;
        }

        if let Some(b) = self.rbr.try_pop() {
            data[0] = b;
        }
    }

    fn out_ier(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 1);
        self.ier = IER::from_bits_truncate(data[0]);
    }

    fn in_ier(&self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        data[0] = self.ier.bits();
    }

    // Divisor Latch Low
    fn out_dll(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 1);
        self.dll = data[0];
    }

    // Divisor Latch Low
    fn in_dll(&self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        data[0] = self.dll;
    }

    // Divisor latch High
    fn out_dlh(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 1);
        self.dlh = data[0];
    }

    // Divisor latch High
    fn in_dlh(&mut self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        data[0] = self.dlh;
    }

    fn in_iir(&mut self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        data[0] = self.iir | (1 << 6) | (1 << 7);
    }

    fn out_fcr(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 1);
        self.fcr = data[0];
    }

    fn out_lcr(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 1);
        self.lcr = LCR::from_bits_truncate(data[0]);
    }

    fn in_lcr(&self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        data[0] = self.lcr.bits();
    }

    fn out_mcr(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 1);
        self.mcr = MCR::from_bits_truncate(data[0]);
    }

    fn in_mcr(&self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        data[0] = self.mcr.bits();
    }

    fn in_lsr(&mut self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        data[0] = self.lsr.bits();
    }

    fn in_msr(&self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        data[0] = self.msr.bits();
    }

    fn out_sr(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 1);
        self.sr = data[0];
    }

    fn in_sr(&self, data: &mut [u8]) {
        assert_eq!(data.len(), 1);
        data[0] = self.sr;
    }

    fn update_state(&mut self) {
        // update by fcr
        {
            if self.fcr & FCR_CLEAR_RECEIVE_FIFO != 0 {
                self.rbr.clean();
                self.fcr &= !FCR_CLEAR_RECEIVE_FIFO;
            }

            if self.fcr & FCR_CLEAR_TRANSMIT_FIFO != 0 {
                self.txr.clean();
                self.fcr &= !FCR_CLEAR_TRANSMIT_FIFO;
            }
        }

        // update LSR
        {
            if self.rbr.is_empty() {
                self.lsr.remove(LSR::DataReady);
            } else {
                self.lsr.insert(LSR::DataReady);
            }

            if self.txr.is_full() {
                self.lsr.remove(LSR::THREmpty);
                self.lsr.remove(LSR::THREmptyAndLineIdle);
            } else {
                self.lsr.insert(LSR::THREmpty);
                self.lsr.insert(LSR::THREmptyAndLineIdle);
            }
        }

        // TODO: Fifo?
        let mut iir = 0;

        if self.ier.contains(IER::ReceiveDataAvailable) && self.lsr.contains(LSR::DataReady) {
            iir |= IIR_RECEIVER_DATA_AVAILABLE;
        }

        if self.ier.contains(IER::TransmitterHoldingRegisterEmpty)
            && self.lsr.contains(LSR::THREmpty)
        {
            iir |= IIR_TRANSMITTTER_HOLDING_REGISTER_EMPTY;
        }

        let has_interrupt = iir != 0;

        if iir == 0 {
            // 1 means no interrupt pending!!!
            iir |= IIR_INTERRUPT_PENDING_FLAG;
        }

        self.iir = iir;

        if has_interrupt != self.irq_state {
            self.irq_controller.trigger_irq(IRQ, has_interrupt);
        }

        self.irq_state = has_interrupt;
    }

    fn receive_byte(&mut self, data: u8) {
        if self.rbr.is_full() {
            warn!("uart rbr is full, data {data} is discarded");
            return;
        }

        self.rbr.push(data).unwrap();

        self.update_state();
    }
}

pub struct Uart8250<const IRQ: u32> {
    port_base: u16,
    internal: Arc<Mutex<Uart8250Internal<IRQ>>>,
}

impl<const IRQ: u32> Uart8250<IRQ> {
    pub fn new(
        pio_allocator: &mut RangeAllocator<u16>,
        port_base: u16,
        irq_controller: Arc<dyn InterruptController>,
        console: bool,
    ) -> Result<Self, DeviceError> {
        let _ = pio_allocator.reserve(port_base, 8)?;

        let internal = Arc::new(Mutex::new(Uart8250Internal {
            txr: Default::default(),
            rbr: Default::default(),
            dll: Default::default(),
            dlh: Default::default(),
            ier: Default::default(),
            iir: Default::default(),
            fcr: Default::default(),
            lcr: Default::default(),
            mcr: Default::default(),
            lsr: Default::default(),
            msr: Default::default(),
            sr: Default::default(),
            irq_controller,
            irq_state: false,
        }));

        if console {
            tokio::spawn({
                let raw = internal.clone();
                async move {
                    let mut stdin = tokio::io::stdin();
                    let mut buffer = [0u8; 1];

                    while let Ok(n) = stdin.read(&mut buffer).await {
                        if n == 0 {
                            break;
                        }
                        let mut raw = raw.lock().await;
                        raw.receive_byte(buffer[0]);
                    }
                }
            });
        }

        Ok(Uart8250 {
            port_base,
            internal,
        })
    }
}

impl<const IRQ: u32> Device for Uart8250<IRQ> {
    fn name(&self) -> String {
        "uart8250".to_string()
    }

    fn support_pio_transport(&self) -> Option<&dyn PioDevice> {
        Some(self)
    }

    fn support_pio_transport_mut(&mut self) -> Option<&mut dyn PioDevice> {
        Some(self)
    }
}

impl<const IRQ: u32> PioDevice for Uart8250<IRQ> {
    fn ports(&self) -> Vec<Range<u16>> {
        let range = self.port_base..self.port_base + 8;
        vec![range]
    }

    fn io_in(&self, port: u16, data: &mut [u8]) -> Result<(), DeviceError> {
        let offset = port - self.port_base;

        let mut internal = self.internal.blocking_lock();

        match offset {
            RBR if !internal.lcr.is_dlab_set() => internal.in_rbr(data),
            DLL if internal.lcr.is_dlab_set() => internal.in_dll(data),
            IER if !internal.lcr.is_dlab_set() => internal.in_ier(data),
            DLH if internal.lcr.is_dlab_set() => internal.in_dlh(data),
            IIR => internal.in_iir(data),
            LCR => internal.in_lcr(data),
            MCR => internal.in_mcr(data),
            LSR => internal.in_lsr(data),
            MSR => internal.in_msr(data),
            SR => internal.in_sr(data),
            _ => unreachable!(),
        }

        internal.update_state();

        Ok(())
    }

    fn io_out(&self, port: u16, data: &[u8]) -> Result<(), DeviceError> {
        let offset = port - self.port_base;

        let mut internal = self.internal.blocking_lock();

        match offset {
            THR if !internal.lcr.is_dlab_set() => internal.out_thr(data),
            DLL if internal.lcr.is_dlab_set() => internal.out_dll(data),
            IER if !internal.lcr.is_dlab_set() => internal.out_ier(data),
            DLH if internal.lcr.is_dlab_set() => internal.out_dlh(data),
            FCR => internal.out_fcr(data),
            LCR => internal.out_lcr(data),
            MCR => internal.out_mcr(data),
            LSR => (), // Ignore
            MSR => (), // Ignore
            SR => internal.out_sr(data),
            _ => unreachable!(),
        }

        internal.update_state();

        Ok(())
    }
}
