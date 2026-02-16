use std::marker::PhantomData;

use vm_core::device::Device;
use vm_core::device::mmio::MmioRange;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_core::device::mmio::mmio_device::MmioHandler;
use vm_fdt::FdtWriter;

use crate::device::VirtIoDevice;
use crate::transport::VirtIoDev;
use crate::transport::mmio::handler::Handler;

mod control_register;

const CONFIGURATION_SPACE_OFFSET: usize = 0x100;

mod handler {
    use tracing::error;
    use tracing::warn;
    use vm_core::device::mmio::MmioRange;
    use vm_core::device::mmio::mmio_device::MmioHandler;

    use crate::device::VirtIoDevice;
    use crate::result::Result as VirtIoResult;
    use crate::transport::VirtIoDev;
    use crate::transport::VirtIoDevInternal;
    use crate::transport::control_register::ControlRegister;
    use crate::transport::mmio::CONFIGURATION_SPACE_OFFSET;
    use crate::transport::mmio::control_register::MmioControlRegister;
    use crate::types::interrupt_status::InterruptStatus;

    pub struct Handler<D> {
        mmio_range: MmioRange,
        transport: VirtIoDev<D>,
    }

    impl<D> Handler<D>
    where
        D: VirtIoDevice,
    {
        pub fn new(mmio_range: MmioRange, transport: VirtIoDev<D>) -> Self {
            Handler {
                mmio_range,
                transport,
            }
        }

        fn read_reg(&self, transport: &VirtIoDevInternal<D>, reg: MmioControlRegister) -> u32 {
            match reg {
                MmioControlRegister::MagicValue => u32::from_le_bytes(*b"virt"),
                MmioControlRegister::Version => 0x2,
                MmioControlRegister::DeviceId => D::DEVICE_ID,
                MmioControlRegister::VendorId => u32::from_le_bytes(*b"QEMU"),
                MmioControlRegister::DeviceFeatures => {
                    transport.read_reg(ControlRegister::DeviceFeatures)
                }
                MmioControlRegister::QueueSizeMax => {
                    transport.read_reg(ControlRegister::QueueSizeMax)
                }
                MmioControlRegister::QueueReady => transport.read_reg(ControlRegister::QueueReady),
                MmioControlRegister::InterruptStatus => {
                    transport.read_reg(ControlRegister::InterruptStatus)
                }
                MmioControlRegister::Status => transport.read_reg(ControlRegister::Status),
                MmioControlRegister::QueueReset => todo!(),
                MmioControlRegister::ConfigGeneration => {
                    transport.read_reg(ControlRegister::ConfigGeneration)
                }
                _ => unreachable!("try to read a WO register: {reg:?}"),
            }
        }

        fn write_reg(
            &self,
            transport: &mut VirtIoDevInternal<D>,
            reg: MmioControlRegister,
            val: u32,
        ) -> VirtIoResult<()> {
            match reg {
                MmioControlRegister::DeviceFeaturesSel => {
                    transport.write_reg(ControlRegister::DeviceFeaturesSel, val)
                }
                MmioControlRegister::DriverFeatures => {
                    transport.write_reg(ControlRegister::DriverFeatures, val)
                }
                MmioControlRegister::DriverFeaturesSel => {
                    transport.write_reg(ControlRegister::DriverFeaturesSel, val)
                }
                MmioControlRegister::QueueSel => {
                    transport.write_reg(ControlRegister::QueueSel, val)
                }
                MmioControlRegister::QueueSize => {
                    transport.write_reg(ControlRegister::QueueSize, val)
                }
                MmioControlRegister::QueueReady => {
                    transport.write_reg(ControlRegister::QueueReady, val)
                }
                MmioControlRegister::QueueNotify => {
                    transport.write_reg(ControlRegister::QueueNotify, val)
                }
                MmioControlRegister::InterruptAck => {
                    transport
                        .interrupt_status
                        .remove(InterruptStatus::from_bits_truncate(val));

                    if transport.interrupt_status.is_empty() {
                        transport.device.trigger_irq(false);
                    }

                    Ok(())
                }
                MmioControlRegister::Status => transport.write_reg(ControlRegister::Status, val),
                MmioControlRegister::QueueDescLow => {
                    transport.write_reg(ControlRegister::QueueDescLow, val)
                }
                MmioControlRegister::QueueDescHigh => {
                    transport.write_reg(ControlRegister::QueueDescHigh, val)
                }
                MmioControlRegister::QueueAvailLow => {
                    transport.write_reg(ControlRegister::QueueAvailLow, val)
                }
                MmioControlRegister::QueueAvailHigh => {
                    transport.write_reg(ControlRegister::QueueAvailHigh, val)
                }
                MmioControlRegister::QueueUsedLow => {
                    transport.write_reg(ControlRegister::QueueUsedLow, val)
                }
                MmioControlRegister::QueueUsedHigh => {
                    transport.write_reg(ControlRegister::QueueUsedHigh, val)
                }
                MmioControlRegister::ShmSel => todo!(),
                MmioControlRegister::QueueReset => todo!(),
                _ => unreachable!("Try to write a RO register {reg:?}"),
            }
        }
    }

    impl<D> MmioHandler for Handler<D>
    where
        D: VirtIoDevice,
    {
        fn mmio_range(&self) -> MmioRange {
            self.mmio_range
        }

        fn mmio_read(&self, offset: u64, len: usize, data: &mut [u8]) {
            let transport = self.transport.blocking_lock();

            let offset: usize = offset.try_into().unwrap();
            if offset < CONFIGURATION_SPACE_OFFSET {
                if let Some(reg) = MmioControlRegister::from_repr(offset as u16) {
                    assert_eq!(len, 4);
                    assert_eq!(data.len(), 4);

                    let val = self.read_reg(&transport, reg);

                    data.copy_from_slice(&val.to_le_bytes());
                } else {
                    warn!(
                        offset,
                        len,
                        ?data,
                        "read from invalid offset of virtio-mmio device"
                    );

                    panic!()
                }
            } else if let Err(err) =
                transport.read_config(offset - CONFIGURATION_SPACE_OFFSET, len, data)
            {
                error!(?err, "Failed to read device configuration");

                panic!()
            }
        }

        fn mmio_write(&self, offset: u64, len: usize, data: &[u8]) {
            let mut transport = self.transport.blocking_lock();

            let offset: usize = offset.try_into().unwrap();
            if offset < CONFIGURATION_SPACE_OFFSET {
                if let Some(reg) = MmioControlRegister::from_repr(offset as u16) {
                    assert_eq!(len, 4);
                    assert_eq!(data.len(), 4);

                    self.write_reg(
                        &mut transport,
                        reg,
                        u32::from_le_bytes(data.try_into().unwrap()),
                    )
                    .unwrap();
                } else {
                    warn!(
                        offset,
                        len,
                        ?data,
                        "write from invalid offset of virtio-mmio device"
                    );

                    panic!()
                }
            } else if let Err(err) =
                transport.write_config(offset - CONFIGURATION_SPACE_OFFSET, len, data)
            {
                error!(?err, "Failed to write device configuration");

                panic!()
            }
        }
    }
}

pub struct VirtIoMmioTransport<D> {
    mmio_range: MmioRange,
    irq: Option<u32>,
    transport: VirtIoDev<D>,
    _mark: PhantomData<D>,
}

impl<D> VirtIoMmioTransport<D>
where
    D: VirtIoDevice,
{
    pub fn new(device: D, mmio_range: MmioRange) -> Self {
        VirtIoMmioTransport {
            mmio_range,
            irq: device.irq(),
            transport: device.into(),
            _mark: PhantomData,
        }
    }

    fn generate_mmio_handler(&self) -> Handler<D> {
        Handler::new(self.mmio_range, self.transport.clone())
    }
}

impl<D> Device for VirtIoMmioTransport<D>
where
    D: VirtIoDevice,
{
    fn name(&self) -> String {
        D::NAME.to_string()
    }
}

impl<D> MmioDevice for VirtIoMmioTransport<D>
where
    D: VirtIoDevice,
{
    fn mmio_range_handlers(&self) -> Vec<Box<dyn MmioHandler>> {
        vec![Box::new(self.generate_mmio_handler())]
    }

    fn generate_dt(&self, fdt: &mut FdtWriter) -> Result<(), vm_fdt::Error> {
        let node = fdt.begin_node(&format!("{}@{:x}", self.name(), self.mmio_range.start))?;

        fdt.property_string("compatible", "virtio,mmio")?;
        fdt.property_array_u64("reg", &[self.mmio_range.start, self.mmio_range.len as u64])?;
        if let Some(_irq) = self.irq {
            #[cfg(target_arch = "aarch64")]
            {
                use vm_core::irq::arch::aarch64::GIC_SPI;
                use vm_core::irq::arch::aarch64::IRQ_TYPE_LEVEL_HIGH;
                fdt.property_array_u32("interrupts", &[GIC_SPI, _irq, IRQ_TYPE_LEVEL_HIGH])?;
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                todo!()
            }
        }

        fdt.end_node(node)?;

        Ok(())
    }
}
