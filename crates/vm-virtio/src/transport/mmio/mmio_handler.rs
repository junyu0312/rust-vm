use tracing::debug;
use tracing::error;
use tracing::warn;
use vm_core::device::mmio::layout::MmioRange;
use vm_core::device::mmio::mmio_device::MmioHandler;
use vm_mm::memory_container::MemoryContainer;

use crate::device::VirtioDevice;
use crate::result::Result as VirtioResult;
use crate::transport::VirtioDev;
use crate::transport::control_register::ControlRegister;
use crate::transport::mmio::VirtioMmioTransport;
use crate::transport::mmio::control_register::MmioControlRegister;
use crate::types::interrupt_status::InterruptStatus;

const CONFIGURATION_SPACE_OFFSET: usize = 0x100;

const VIRTIO_MMIO_MAGIC_VALUE: u32 = u32::from_le_bytes(*b"virt");
const VIRTIO_MMIO_VERSION: u32 = 0x2;
const VIRTIO_MMIO_VENDOR_ID: u32 = u32::from_le_bytes(*b"QEMU");

impl<C, D> VirtioMmioTransport<C, D>
where
    D: VirtioDevice<C>,
{
    fn read_reg(&self, dev: &VirtioDev<C, D>, reg: MmioControlRegister) -> u32 {
        match reg {
            MmioControlRegister::MagicValue => VIRTIO_MMIO_MAGIC_VALUE,
            MmioControlRegister::Version => VIRTIO_MMIO_VERSION,
            MmioControlRegister::DeviceId => D::DEVICE_ID as u32,
            MmioControlRegister::VendorId => VIRTIO_MMIO_VENDOR_ID,
            MmioControlRegister::DeviceFeatures => dev.read_reg(ControlRegister::DeviceFeatures),
            MmioControlRegister::QueueSizeMax => dev.read_reg(ControlRegister::QueueSizeMax),
            MmioControlRegister::QueueReady => dev.read_reg(ControlRegister::QueueReady),
            MmioControlRegister::InterruptStatus => dev.read_reg(ControlRegister::InterruptStatus),
            MmioControlRegister::Status => dev.read_reg(ControlRegister::Status),
            MmioControlRegister::QueueReset => todo!(),
            MmioControlRegister::ConfigGeneration => {
                dev.read_reg(ControlRegister::ConfigGeneration)
            }
            _ => {
                warn!(?reg, "try to read a WO register");
                0 // ignore the error
            }
        }
    }

    fn write_reg(
        &self,
        dev: &mut VirtioDev<C, D>,
        reg: MmioControlRegister,
        val: u32,
    ) -> VirtioResult<()> {
        match reg {
            MmioControlRegister::DeviceFeaturesSel => {
                dev.write_reg(ControlRegister::DeviceFeaturesSel, val)
            }
            MmioControlRegister::DriverFeatures => {
                dev.write_reg(ControlRegister::DriverFeatures, val)
            }
            MmioControlRegister::DriverFeaturesSel => {
                dev.write_reg(ControlRegister::DriverFeaturesSel, val)
            }
            MmioControlRegister::QueueSel => dev.write_reg(ControlRegister::QueueSel, val),
            MmioControlRegister::QueueSize => dev.write_reg(ControlRegister::QueueSize, val),
            MmioControlRegister::QueueReady => dev.write_reg(ControlRegister::QueueReady, val),
            MmioControlRegister::QueueNotify => dev.write_reg(ControlRegister::QueueNotify, val),
            MmioControlRegister::InterruptAck => {
                let mut is = dev.get_interrupt_status();
                is.remove(InterruptStatus::from_bits_truncate(val));
                dev.update_interrupt_status(is);

                Ok(())
            }
            MmioControlRegister::Status => dev.write_reg(ControlRegister::Status, val),
            MmioControlRegister::QueueDescLow => dev.write_reg(ControlRegister::QueueDescLow, val),
            MmioControlRegister::QueueDescHigh => {
                dev.write_reg(ControlRegister::QueueDescHigh, val)
            }
            MmioControlRegister::QueueAvailLow => {
                dev.write_reg(ControlRegister::QueueAvailLow, val)
            }
            MmioControlRegister::QueueAvailHigh => {
                dev.write_reg(ControlRegister::QueueAvailHigh, val)
            }
            MmioControlRegister::QueueUsedLow => dev.write_reg(ControlRegister::QueueUsedLow, val),
            MmioControlRegister::QueueUsedHigh => {
                dev.write_reg(ControlRegister::QueueUsedHigh, val)
            }
            MmioControlRegister::ShmSel => todo!(),
            MmioControlRegister::QueueReset => todo!(),
            _ => {
                warn!(name = D::NAME, ?reg, "Try to write a RO register");
                Ok(()) // ignore the error
            }
        }
    }
}

impl<C, D> MmioHandler for VirtioMmioTransport<C, D>
where
    C: MemoryContainer,
    D: VirtioDevice<C>,
{
    fn mmio_range(&self) -> MmioRange {
        self.mmio_range
    }

    fn mmio_read(&self, offset: u64, len: usize, data: &mut [u8]) {
        let dev = self.dev.lock().unwrap();

        let Ok(offset) = usize::try_from(offset) else {
            warn!(name = D::NAME, offset, "offset too large");
            return;
        };
        if offset < CONFIGURATION_SPACE_OFFSET {
            if let Some(reg) = MmioControlRegister::from_repr(offset as u16) {
                assert_eq!(len, data.len()); // TODO: mmio_read can remove the `len` argument in the future
                if data.len() == 4 {
                    let val = self.read_reg(&dev, reg);

                    debug!(name = D::NAME, ?reg, len, val, "virtio-mmio read");

                    data.copy_from_slice(&val.to_le_bytes());
                } else {
                    warn!(name = D::NAME, ?reg, len, "invalid virtio-mmio access size");
                    debug_assert!(false);
                }
            } else {
                warn!(
                    name = D::NAME,
                    offset,
                    len,
                    ?data,
                    "read from invalid offset of the virtio-mmio device"
                );

                debug_assert!(false)
            }
        } else if let Err(err) = dev.read_config(offset - CONFIGURATION_SPACE_OFFSET, data) {
            error!(name = D::NAME, ?err, "Failed to read device configuration");

            debug_assert!(false)
        }
    }

    fn mmio_write(&self, offset: u64, len: usize, data: &[u8]) {
        let mut dev = self.dev.lock().unwrap();

        debug!(name = D::NAME, offset, len, ?data, "virtio-mmio write");

        let Ok(offset) = usize::try_from(offset) else {
            warn!(name = D::NAME, offset, "offset too large");
            return;
        };
        if offset < CONFIGURATION_SPACE_OFFSET {
            if let Some(reg) = MmioControlRegister::from_repr(offset as u16) {
                assert_eq!(len, data.len());
                if data.len() == 4 {
                    if let Err(err) =
                        self.write_reg(&mut dev, reg, u32::from_le_bytes(data.try_into().unwrap()))
                    {
                        error!(
                            name = D::NAME,
                            ?err,
                            offset,
                            len,
                            ?data,
                            "error while writing virtio-mmio control register"
                        );

                        debug_assert!(false)
                    }
                } else {
                    warn!(name = D::NAME, ?reg, len, "invalid virtio-mmio access size");
                    debug_assert!(false);
                }
            } else {
                warn!(
                    name = D::NAME,
                    offset,
                    len,
                    ?data,
                    "write to invalid offset of the virtio-mmio device"
                );

                debug_assert!(false)
            }
        } else if let Err(err) = dev.write_config(offset - CONFIGURATION_SPACE_OFFSET, data) {
            error!(name = D::NAME, ?err, "Failed to write device configuration");

            debug_assert!(false)
        }
    }
}
