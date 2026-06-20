use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::ops::Range;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

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
use tokio::runtime::Handle;
use vm_core::arch::irq::InterruptController;
use vm_core::device::Device;
use vm_core::device::error::DeviceError;
use vm_core::device::error::DeviceSnapshotError;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_fdt::FdtWriter;
use vm_mm::manager::MemoryAddressSpace;

use crate::device::VirtioDevice;
use crate::device::virtqueue::VirtioConfigurationChangeNotifier;
use crate::device::virtqueue::VirtioUsedBufferNotifier;
use crate::transport::VirtioDeviceOps;
use crate::transport::common::VirtioTransportCommon;
use crate::transport::common::VirtqueueHandler;
use crate::transport::mmio::interrupt::VirtioMmioEventNotifier;

mod control_register;
mod interrupt;
mod mmio_handler;

pub struct VirtioMmioTransport<D> {
    // a unique index for building acpi
    virtio_mmio_device_index: u8,
    mmio_range: Range<u64>,
    irq: u8,
    common: Mutex<VirtioTransportCommon<D>>,

    tokio_runtime: Handle,
    memory: Arc<MemoryAddressSpace>,
    irq_chip: Arc<dyn InterruptController>,

    virtqueue_handlers: RwLock<HashMap<u16, VirtqueueHandler>>,
    event_notification: Arc<VirtioMmioEventNotifier>,
}

impl<D> VirtioMmioTransport<D>
where
    D: VirtioDevice,
{
    pub fn new(
        tokio_runtime: Handle,
        memory: Arc<MemoryAddressSpace>,
        irq_chip: Arc<dyn InterruptController>,
        virtio_mmio_device_index: u8,
        mmio_range: Range<u64>,
        irq: u8,
        common: VirtioTransportCommon<D>,
    ) -> Self {
        let event_notification = Arc::new(VirtioMmioEventNotifier::new(
            irq_chip.clone(),
            irq as u32,
            common.get_interrupt_status(),
            common.get_config_generation(),
        ));

        VirtioMmioTransport {
            virtio_mmio_device_index,
            mmio_range,
            irq,
            common: Mutex::new(common),

            tokio_runtime,
            memory,
            irq_chip,

            virtqueue_handlers: Default::default(),
            event_notification,
        }
    }

    pub(crate) fn get_used_buffer_notification(&self) -> Arc<dyn VirtioUsedBufferNotifier> {
        self.event_notification.clone()
    }

    pub(crate) fn get_configuration_change_notification(
        &self,
    ) -> Arc<dyn VirtioConfigurationChangeNotifier> {
        self.event_notification.clone()
    }
}

impl<D> Device for VirtioMmioTransport<D>
where
    D: VirtioDevice,
{
    fn name(&self) -> String {
        D::NAME.to_string()
    }

    fn pause(&self) -> std::result::Result<(), DeviceSnapshotError> {
        self.common.lock().unwrap().pause()
    }

    fn resume(&self) -> std::result::Result<(), DeviceSnapshotError> {
        self.common.lock().unwrap().resume()
    }

    fn save(&self, writer: &mut dyn Write) -> std::result::Result<(), DeviceSnapshotError> {
        self.common.lock().unwrap().save(writer)
    }

    fn load(&mut self, reader: &mut dyn Read) -> std::result::Result<(), DeviceSnapshotError> {
        self.common.lock().unwrap().load(reader)
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

impl<D> Aml for VirtioMmioTransport<D>
where
    D: VirtioDevice,
{
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        AmlDevice::new(
            Path::new(&format!("V{:03}", self.virtio_mmio_device_index)),
            vec![
                &Name::new("_HID".into(), &"LNRO0005"),
                &Name::new("_UID".into(), &self.virtio_mmio_device_index),
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
                        &Interrupt::new(true, true, false, false, self.irq as u32),
                    ]),
                ),
            ],
        )
        .to_aml_bytes(sink);
    }
}

impl<D> MmioDevice for VirtioMmioTransport<D>
where
    D: VirtioDevice,
{
    fn mmio_ranges(&self) -> Vec<Range<u64>> {
        vec![self.mmio_range.clone()]
    }

    fn mmio_read(&self, addr: u64, buf: &mut [u8]) -> std::result::Result<(), DeviceError> {
        debug_assert!(self.mmio_range.contains(&addr));

        self.read(addr - self.mmio_range.start, buf)
            .map_err(|err| DeviceError::Device(Box::new(err)))
    }

    fn mmio_write(&self, addr: u64, buf: &[u8]) -> std::result::Result<(), DeviceError> {
        debug_assert!(self.mmio_range.contains(&addr));

        self.write(addr - self.mmio_range.start, buf)
            .map_err(|err| DeviceError::Device(Box::new(err)))
    }

    fn generate_dt(&self, fdt: &mut FdtWriter) -> std::result::Result<(), DeviceError> {
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
            fdt.property_array_u32(
                "interrupts",
                &[GIC_SPI, self.irq as u32, IRQ_TYPE_LEVEL_HIGH],
            )?;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            fdt.property_array_u32("interrupts", &[self.irq as u32, 0])?;
        }

        fdt.end_node(node)?;

        Ok(())
    }
}

impl<D> VirtioDeviceOps for VirtioMmioTransport<D>
where
    D: VirtioDevice,
{
    fn configuration_change_notifier(&self) -> Arc<dyn VirtioConfigurationChangeNotifier> {
        self.get_configuration_change_notification()
    }
}
