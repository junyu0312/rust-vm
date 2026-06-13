use std::io::Read;
use std::io::Write;
use std::ops::Range;
use std::sync::Arc;
use std::sync::Mutex;

use acpi_tables::Aml;
use acpi_tables::AmlSink;
use acpi_tables::aml::AddressSpace;
use acpi_tables::aml::AddressSpaceCacheable;
use acpi_tables::aml::Device as AmlDevice;
use acpi_tables::aml::Interrupt;
use acpi_tables::aml::Name;
use acpi_tables::aml::ONE;
use acpi_tables::aml::Path;
use acpi_tables::aml::ResourceTemplate;
use vm_core::device::Device;
use vm_core::device::error::DeviceError;
use vm_core::device::error::DeviceSnapshotError;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_fdt::FdtWriter;
use vm_utils::range_allocator::RangeAllocator;

use crate::device::VirtioDevice;
use crate::transport::VirtioDev;

mod control_register;
mod mmio_handler;

pub struct VirtioMmioTransport<D> {
    virtio_mmio_index: u8,
    mmio_range: Range<u64>,
    dev: Arc<Mutex<VirtioDev<D>>>,
}

impl<D> VirtioMmioTransport<D>
where
    D: VirtioDevice,
{
    pub fn new(
        mmio_range: Range<u64>,
        virtio_mmio_index: u8,
        dev: Arc<Mutex<VirtioDev<D>>>,
    ) -> Self {
        VirtioMmioTransport {
            virtio_mmio_index,
            mmio_range,
            dev,
        }
    }
}

impl<D> Aml for VirtioMmioTransport<D>
where
    D: VirtioDevice,
{
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        let dev = self.dev.lock().unwrap();

        AmlDevice::new(
            Path::new(&format!("V{:03}", self.virtio_mmio_index)),
            vec![
                &Name::new("_HID".into(), &"LNRO0005"),
                &Name::new("_UID".into(), &self.virtio_mmio_index),
                &Name::new("_CCA".into(), &ONE),
                &Name::new(
                    "_CRS".into(),
                    &ResourceTemplate::new(vec![
                        &AddressSpace::new_memory(
                            AddressSpaceCacheable::NotCacheable,
                            true,
                            self.mmio_range.start,
                            self.mmio_range.end - 1,
                            None,
                        ),
                        &Interrupt::new(true, true, false, false, dev.device.irq()),
                    ]),
                ),
            ],
        )
        .to_aml_bytes(sink);
    }
}

impl<D> Device for VirtioMmioTransport<D>
where
    D: VirtioDevice,
{
    fn name(&self) -> String {
        D::NAME.to_string()
    }

    fn pause(&self) -> Result<(), DeviceSnapshotError> {
        self.dev.lock().unwrap().pause()
    }

    fn save(&self, writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        self.dev.lock().unwrap().save(writer)
    }

    fn load(&mut self, reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        self.dev.lock().unwrap().load(reader)
    }

    fn support_aml(&self) -> Option<&dyn Aml> {
        Some(self)
    }

    fn support_mmio_transport(&self) -> Option<&dyn MmioDevice> {
        Some(self)
    }

    fn support_mmio_transport_mut(&mut self) -> Option<&mut dyn MmioDevice> {
        Some(self)
    }
}

impl<D> MmioDevice for VirtioMmioTransport<D>
where
    D: VirtioDevice,
{
    fn mmio_ranges(&self) -> Vec<Range<u64>> {
        vec![self.mmio_range.clone()]
    }

    fn mmio_read(&self, addr: u64, buf: &mut [u8]) -> Result<(), DeviceError> {
        debug_assert!(self.mmio_range.contains(&addr));

        self.read(addr - self.mmio_range.start, buf);

        Ok(())
    }

    fn mmio_write(&self, addr: u64, buf: &[u8]) -> Result<(), DeviceError> {
        debug_assert!(self.mmio_range.contains(&addr));

        self.write(addr - self.mmio_range.start, buf);

        Ok(())
    }

    fn generate_dt(&self, fdt: &mut FdtWriter) -> Result<(), DeviceError> {
        let dev = self.dev.lock().unwrap();

        let irq = dev.device.irq();

        let node = fdt.begin_node(&format!("{}@{:x}", self.name(), self.mmio_range.start))?;

        fdt.property_string("compatible", "virtio,mmio")?;
        fdt.property_array_u64(
            "reg",
            &[
                self.mmio_range.start,
                self.mmio_range.end - self.mmio_range.start,
            ],
        )?;

        #[cfg(target_arch = "aarch64")]
        {
            use vm_core::arch::aarch64::irq::GIC_SPI;
            use vm_core::arch::aarch64::irq::IRQ_TYPE_LEVEL_HIGH;
            fdt.property_array_u32("interrupts", &[GIC_SPI, irq, IRQ_TYPE_LEVEL_HIGH])?;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            fdt.property_array_u32("interrupts", &[irq, 0])?;
        }

        fdt.end_node(node)?;

        Ok(())
    }
}

pub trait VirtioMmioDevice: VirtioDevice {
    fn into_mmio_device(
        self,
        mmio_allocator: &mut RangeAllocator<u64>,
        virtio_aml_path_allocator: &mut RangeAllocator<u8>,
    ) -> Result<VirtioMmioTransport<Self>, DeviceError> {
        let mmio_range = mmio_allocator
            .alloc(0x1000)
            .map_err(|_| DeviceError::AllocResource)?;

        let id = virtio_aml_path_allocator
            .alloc(1)
            .map_err(|_| DeviceError::AllocResource)?;

        let dev = VirtioMmioTransport::new(mmio_range, id.start, VirtioDev::new(self));

        Ok(dev)
    }
}

impl<T> VirtioMmioDevice for T where T: VirtioDevice {}
