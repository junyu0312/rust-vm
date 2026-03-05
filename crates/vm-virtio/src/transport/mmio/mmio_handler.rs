use std::sync::Arc;
use std::sync::Mutex;

use tracing::debug;
use tracing::error;
use tracing::warn;
use vm_core::device::mmio::MmioRange;
use vm_core::device::mmio::mmio_device::MmioHandler;
use vm_mm::allocator::MemoryContainer;

use crate::device::VirtioDevice;
use crate::result::Result as VirtioResult;
use crate::transport::VirtioDev;
use crate::transport::control_register::ControlRegister;
use crate::transport::mmio::control_register::MmioControlRegister;
use crate::types::interrupt_status::InterruptStatus;

const CONFIGURATION_SPACE_OFFSET: usize = 0x100;

pub struct Handler<C, D> {
    mmio_range: MmioRange,
    transport: Arc<Mutex<VirtioDev<C, D>>>,
}

impl<C, D> Handler<C, D>
where
    D: VirtioDevice<C>,
{
    pub fn new(mmio_range: MmioRange, transport: Arc<Mutex<VirtioDev<C, D>>>) -> Self {
        Handler {
            mmio_range,
            transport,
        }
    }

    fn read_reg(&self, transport: &VirtioDev<C, D>, reg: MmioControlRegister) -> u32 {
        match reg {
            MmioControlRegister::MagicValue => u32::from_le_bytes(*b"virt"),
            MmioControlRegister::Version => 0x2,
            MmioControlRegister::DeviceId => D::DEVICE_ID,
            MmioControlRegister::VendorId => u32::from_le_bytes(*b"QEMU"),
            MmioControlRegister::DeviceFeatures => {
                transport.read_reg(ControlRegister::DeviceFeatures)
            }
            MmioControlRegister::QueueSizeMax => transport.read_reg(ControlRegister::QueueSizeMax),
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
        transport: &mut VirtioDev<C, D>,
        reg: MmioControlRegister,
        val: u32,
    ) -> VirtioResult<()> {
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
            MmioControlRegister::QueueSel => transport.write_reg(ControlRegister::QueueSel, val),
            MmioControlRegister::QueueSize => transport.write_reg(ControlRegister::QueueSize, val),
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

impl<C, D> MmioHandler for Handler<C, D>
where
    C: MemoryContainer,
    D: VirtioDevice<C>,
{
    fn mmio_range(&self) -> MmioRange {
        self.mmio_range
    }

    fn mmio_read(&self, offset: u64, len: usize, data: &mut [u8]) {
        let transport = self.transport.lock().unwrap();

        let offset: usize = offset.try_into().unwrap();
        if offset < CONFIGURATION_SPACE_OFFSET {
            if let Some(reg) = MmioControlRegister::from_repr(offset as u16) {
                assert_eq!(len, 4);
                assert_eq!(data.len(), 4);

                let val = self.read_reg(&transport, reg);

                debug!(name = D::NAME, ?reg, len, val, "read reg from virtio-mmio");

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
        let mut transport = self.transport.lock().unwrap();

        debug!(name = D::NAME, offset, len, ?data);

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
