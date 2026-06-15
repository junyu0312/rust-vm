use std::sync::Mutex;

use vfio_bindings::bindings::vfio::VFIO_PCI_BAR0_REGION_INDEX;
use vm_pci::types::configuration_space::ConfigurationSpace;
use vm_pci::types::configuration_space::header::type0::Type0Header;
use vm_pci::types::function::EcamUpdateCallback;
use vm_pci::types::function::PciFunction;
use vm_pci::types::function::PciFunctionArch;
use vm_pci::types::function::type0::Type0HeaderOffset;
use vm_pci::types::interrupt::InterruptMapEntry;

use crate::vfio::device::VfioDevice;

#[derive(Debug)]
pub enum VfioBarResource {
    Pio,
    Mmio { is_64bit: bool },
}

#[derive(Debug)]
pub struct VfioBarInfo {
    pub(crate) size: u64,
    #[allow(dead_code)]
    pub(crate) resource: VfioBarResource,
}

pub struct VfioPciFunction {
    configuration_space: Mutex<ConfigurationSpace>,
    bars: [Option<VfioBarInfo>; 6],
    device: VfioDevice,
}

impl VfioPciFunction {
    pub(crate) fn new(
        configuration_space: ConfigurationSpace,
        bars: [Option<VfioBarInfo>; 6],
        device: VfioDevice,
    ) -> Self {
        VfioPciFunction {
            configuration_space: configuration_space.into(),
            bars,
            device,
        }
    }

    fn write_bar(&self, bar_index: usize, buf: &[u8]) -> Option<EcamUpdateCallback> {
        let mut configuration_space = self.configuration_space.lock().unwrap();
        let header = configuration_space.as_header_mut::<Type0Header>();

        let data = u32::from_le_bytes(buf.try_into().unwrap());

        if data == u32::MAX {
            let size = if let Some(bar_info) = &self.bars[bar_index] {
                bar_info.size as u32
            } else if bar_index > 0
                && let Some(bar_info) = &self.bars[bar_index - 1]
                && let VfioBarResource::Mmio { is_64bit: true } = bar_info.resource
            {
                (bar_info.size >> 32) as u32
            } else {
                0
            };

            header.bar[bar_index] = !(size.wrapping_sub(1));
            None
        } else {
            header.bar[bar_index] = data;
            self.bars[bar_index]
                .as_ref()
                .map(|bar| EcamUpdateCallback::UpdateMmioRouter {
                    bar: bar_index as u8,
                    pci_address_range: (data as u64)..(data as u64 + bar.size),
                })
        }
    }
}

impl PciFunctionArch for VfioPciFunction {
    fn interrupt_map_entry(
        &self,
        _bus: u8,
        _device: u8,
        _function: u8,
    ) -> Option<InterruptMapEntry> {
        todo!()
    }
}

impl PciFunction for VfioPciFunction {
    fn ecam_read(&self, offset: u16, buf: &mut [u8]) {
        self.configuration_space.lock().unwrap().read(offset, buf);
    }

    fn ecam_write(&self, offset: u16, buf: &[u8]) -> Option<EcamUpdateCallback> {
        match Type0HeaderOffset::from_repr(offset) {
            Some(Type0HeaderOffset::Bar0) => self.write_bar(0, buf),
            Some(Type0HeaderOffset::Bar1) => self.write_bar(1, buf),
            Some(Type0HeaderOffset::Bar2) => self.write_bar(2, buf),
            Some(Type0HeaderOffset::Bar3) => self.write_bar(3, buf),
            Some(Type0HeaderOffset::Bar4) => self.write_bar(4, buf),
            Some(Type0HeaderOffset::Bar5) => self.write_bar(5, buf),
            _ => {
                self.configuration_space.lock().unwrap().write(offset, buf);
                None
            }
        }
    }

    fn bar_read(&self, bar: u8, offset: u64, buf: &mut [u8]) {
        self.device
            .region_read(VFIO_PCI_BAR0_REGION_INDEX + bar as u32, buf, offset)
            .unwrap();
    }

    fn bar_write(&self, bar: u8, offset: u64, buf: &[u8]) {
        self.device
            .region_write(VFIO_PCI_BAR0_REGION_INDEX + bar as u32, buf, offset)
            .unwrap();
    }

    fn legacy_irq(&self) -> Option<(u8, u8)> {
        let cs = self.configuration_space.lock().unwrap();
        let header = cs.as_header::<Type0Header>();

        Some((header.interrupt_pin, header.interrupt_line))
    }
}
