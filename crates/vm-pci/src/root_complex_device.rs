use std::io::Read;
use std::io::Write;
use std::ops::Range;
use std::sync::Arc;
use std::sync::RwLock;

use acpi_tables::Aml;
use acpi_tables::AmlSink;
use acpi_tables::aml::AddressSpace;
use acpi_tables::aml::AddressSpaceCacheable;
use acpi_tables::aml::Device as AmlDevice;
use acpi_tables::aml::Memory32Fixed;
use acpi_tables::aml::Name;
use acpi_tables::aml::Package;
use acpi_tables::aml::ResourceTemplate;
use vm_core::device::Device;
use vm_core::device::error::DeviceError;
use vm_core::device::error::DeviceSnapshotError;
use vm_core::device::mmio::mmio_device::MmioDevice;
#[cfg(target_arch = "x86_64")]
use vm_core::device::pio::pio_device::PioDevice;
use vm_utils::range_allocator::RangeAllocator;

use crate::root_complex::pci_root_complex::PciRootComplex;
use crate::root_complex_device::mmio::MmioTransport;
#[cfg(target_arch = "x86_64")]
use crate::root_complex_device::pio::PioTransport;
use crate::types::device::PciDevice;

mod mmio;
#[cfg(target_arch = "x86_64")]
mod pio;

pub struct PciRootComplexDevice {
    #[cfg(target_arch = "x86_64")]
    pio_transport: PioTransport,
    mmio_transport: MmioTransport,
    internal: Arc<RwLock<PciRootComplex>>,
}

impl PciRootComplexDevice {
    pub fn new(
        #[cfg(target_arch = "x86_64")] pio_allocator: &mut RangeAllocator<u16>,
        mmio_allocator: &mut RangeAllocator<u64>,
        #[cfg(target_arch = "x86_64")] io_port_window: Range<u16>,
        ecam_range: Range<u64>,
        bar_mmio_window: Range<u64>,
    ) -> Result<Self, DeviceError> {
        let internal = Arc::new(RwLock::new(PciRootComplex::default()));
        let device = PciRootComplexDevice {
            #[cfg(target_arch = "x86_64")]
            pio_transport: PioTransport::new(
                pio_allocator,
                #[cfg(target_arch = "x86_64")]
                io_port_window,
                internal.clone(),
            )?,
            mmio_transport: MmioTransport::new(
                mmio_allocator,
                ecam_range,
                bar_mmio_window,
                internal.clone(),
            )?,
            internal,
        };

        Ok(device)
    }

    pub fn register_device(
        &mut self,
        device: Box<dyn PciDevice>,
    ) -> Result<(), Box<dyn PciDevice>> {
        self.internal.write().unwrap().register_device(device)
    }
}

impl Aml for PciRootComplexDevice {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        let internal = self.internal.read().unwrap();

        assert_eq!(internal.bus.len(), 1, "we only support one bus(0) yet.");

        let mut address_irq = Vec::new();
        for (device_id, device) in internal.bus[0].devices() {
            for (function_id, function) in device.functions().enumerate() {
                let address = ((*device_id as u32) << 16) | (function_id as u32);

                if let Some((line, pin)) = function.legacy_irq() {
                    // Pci header: 0x01 -> IntA
                    // Acpi: 0x00 -> IntA
                    address_irq.push((address, line as u32, pin - 1));
                }
            }
        }
        let prt = address_irq
            .iter()
            .map(|(address, line, pin)| {
                assert!(*pin <= 3);
                // (address, pin, source, source index)
                Package::new(vec![address, pin, &0u8, line])
            })
            .collect::<Vec<Package>>();
        let prt = prt.iter().map(|aml| aml as _).collect();

        AmlDevice::new(
            "_SB_.PCI0".into(),
            vec![
                &Name::new("_HID".into(), &"PNP0A08"),
                &Name::new("_CID".into(), &"PNP0A03"),
                &Name::new(
                    "_CRS".into(),
                    &ResourceTemplate::new(vec![
                        &AddressSpace::new_bus_number(0u16, 0u16),
                        &Memory32Fixed::new(
                            true,
                            self.mmio_transport.ecam_range.start.try_into().unwrap(),
                            (self.mmio_transport.ecam_range.end
                                - self.mmio_transport.ecam_range.start)
                                .try_into()
                                .unwrap(),
                        ),
                        &AddressSpace::new_memory(
                            AddressSpaceCacheable::NotCacheable,
                            true,
                            self.mmio_transport.pci_bar_mmio_window.start,
                            self.mmio_transport.pci_bar_mmio_window.end - 1,
                            None,
                        ),
                        #[cfg(target_arch = "x86_64")]
                        &AddressSpace::new_io(
                            self.pio_transport.io_port_window.start,
                            self.pio_transport.io_port_window.end,
                            None,
                        ),
                    ]),
                ),
                &Name::new("_PRT".into(), &Package::new(prt)),
            ],
        )
        .to_aml_bytes(sink);
    }
}

impl Device for PciRootComplexDevice {
    fn name(&self) -> String {
        "pci-root-complex".to_string()
    }

    fn save(&self, writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        self.internal.read().unwrap().save(writer)
    }

    fn load(&mut self, read: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        self.internal.write().unwrap().load(read)
    }

    fn support_aml(&self) -> Option<&dyn Aml> {
        Some(self)
    }

    #[cfg(target_arch = "x86_64")]
    fn support_pio_transport(&self) -> Option<&dyn PioDevice> {
        Some(&self.pio_transport)
    }

    #[cfg(target_arch = "x86_64")]
    fn support_pio_transport_mut(&mut self) -> Option<&mut dyn PioDevice> {
        Some(&mut self.pio_transport)
    }

    fn support_mmio_transport(&self) -> Option<&dyn MmioDevice> {
        Some(&self.mmio_transport)
    }

    fn support_mmio_transport_mut(&mut self) -> Option<&mut dyn MmioDevice> {
        Some(&mut self.mmio_transport)
    }
}
