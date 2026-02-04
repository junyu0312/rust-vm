use strum_macros::FromRepr;
use tracing::debug;
use tracing::error;
use tracing::warn;
use vm_core::device::Device;
use vm_core::device::mmio::MmioDevice;
use vm_core::device::mmio::MmioRange;
use vm_fdt::FdtWriter;

use crate::transport::Result as VirtIoResult;
use crate::transport::VirtIo;
use crate::transport::VirtIoError;
use crate::types::device::Subsystem;

const DEVICE_SPECIFIC_CONFIGURATION_OFFSET: usize = 0x100;

#[derive(Debug, FromRepr)]
#[repr(u16)]
pub enum ControlRegister {
    /* Control registers */
    /// Magic value ("virt") - Read Only
    MagicValue = 0x000,

    /// Virtio device version - Read Only
    Version = 0x004,

    /// Virtio device ID - Read Only
    DeviceId = 0x008,

    /// Virtio vendor ID - Read Only
    VendorId = 0x00c,

    /// Device features (host) - Read Only
    DeviceFeatures = 0x010,

    /// Device features selector - Write Only
    DeviceFeaturesSel = 0x014,

    /// Driver features (guest) - Write Only
    DriverFeatures = 0x020,

    /// Driver features selector - Write Only
    DriverFeaturesSel = 0x024,

    /// Queue selector - Write Only
    QueueSel = 0x030,

    /// Maximum queue size - Read Only
    QueueSizeMax = 0x034,

    /// Queue size - Write Only
    QueueSize = 0x038,

    /// Queue ready - Read Write
    QueueReady = 0x044,

    /// Queue notify - Write Only
    QueueNotify = 0x050,

    /// Interrupt status - Read Only
    InterruptStatus = 0x060,

    /// Interrupt acknowledge - Write Only
    InterruptAck = 0x064,

    /// Device status - Read Write
    Status = 0x070,

    /// Descriptor table address (low 32 bits)
    QueueDescLow = 0x080,

    /// Descriptor table address (high 32 bits)
    QueueDescHigh = 0x084,

    /// Available ring address (low 32 bits)
    QueueAvailLow = 0x090,

    /// Available ring address (high 32 bits)
    QueueAvailHigh = 0x094,

    /// Used ring address (low 32 bits)
    QueueUsedLow = 0x0a0,

    /// Used ring address (high 32 bits)
    QueueUsedHigh = 0x0a4,

    /// Shared memory region selector
    ShmSel = 0x0ac,

    /// Shared memory length (low 32 bits)
    ShmLenLow = 0x0b0,

    /// Shared memory length (high 32 bits)
    ShmLenHigh = 0x0b4,

    /// Shared memory base address (low 32 bits)
    ShmBaseLow = 0x0b8,

    /// Shared memory base address (high 32 bits)
    ShmBaseHigh = 0x0bc,

    /// Configuration generation
    ConfigGeneration = 0x0fc,
}

pub trait VirtIoMmio: VirtIo {
    fn mmio_range(&self) -> MmioRange;

    fn interrupts(&self) -> Option<&[u32]>;

    fn write_control_register(&mut self, reg: ControlRegister, val: u32) -> VirtIoResult<()> {
        debug!(name = Self::NAME, ?reg, val, "write");

        match reg {
            ControlRegister::DeviceFeaturesSel => self.write_device_feature_sel(val),
            ControlRegister::DriverFeatures => self.write_driver_features(val),
            ControlRegister::DriverFeaturesSel => self.write_driver_feature_sel(val),
            ControlRegister::QueueSel => self.write_queue_sel(val),
            ControlRegister::QueueSize => self.write_queue_size(val.try_into().unwrap()),
            ControlRegister::QueueReady => self.write_queue_ready(val != 0),
            ControlRegister::QueueNotify => self.write_queue_notify(val),
            ControlRegister::InterruptAck => self.write_interrupt_ack(val),
            ControlRegister::Status => {
                self.write_status(val.try_into().map_err(|_| VirtIoError::InvalidFlagLen)?)
            }
            ControlRegister::QueueDescLow => self.write_queue_desc_low(val),
            ControlRegister::QueueDescHigh => self.write_queue_desc_high(val),
            ControlRegister::QueueAvailLow => self.write_queue_driver_low(val),
            ControlRegister::QueueAvailHigh => self.write_queue_driver_high(val),
            ControlRegister::QueueUsedLow => self.write_queue_device_low(val),
            ControlRegister::QueueUsedHigh => self.write_queue_device_high(val),
            ControlRegister::ShmSel => todo!(),
            ControlRegister::ShmBaseLow => todo!(),
            ControlRegister::ShmBaseHigh => todo!(),
            _ => unreachable!("Try to write a read-only register {reg:?}"),
        }

        Ok(())
    }

    fn read_control_register(&mut self, reg: ControlRegister) -> VirtIoResult<u32> {
        let v = match reg {
            ControlRegister::MagicValue => 0x74726976, // string `virt`
            ControlRegister::Version => 0x2,           // only support new version
            ControlRegister::DeviceId => Self::Subsystem::DEVICE_ID,
            ControlRegister::VendorId => 0x554d4551, // string `QEMU`
            ControlRegister::DeviceFeatures => self.read_device_features(),
            ControlRegister::QueueSizeMax => self.read_queue_size_max(),
            ControlRegister::QueueReady => self.read_queue_ready() as u32,
            ControlRegister::InterruptStatus => self.read_interrupt_status(),
            ControlRegister::Status => self.read_status().as_u32(),
            ControlRegister::ShmLenLow => todo!(),
            ControlRegister::ShmLenHigh => todo!(),
            ControlRegister::ShmBaseLow => todo!(),
            ControlRegister::ShmBaseHigh => todo!(),
            ControlRegister::ConfigGeneration => self.read_config_generation(),
            _ => unreachable!("Try to read a write-only register {reg:?}"),
        };

        debug!(name = Self::NAME, ?reg, v, "read");

        Ok(v)
    }

    fn generate_dt(&self, fdt: &mut FdtWriter) -> Result<(), vm_fdt::Error> {
        let node = fdt.begin_node(&format!("{}@{:x}", Self::NAME, self.mmio_range().start))?;

        fdt.property_string("compatible", "virtio,mmio")?;
        fdt.property_array_u64(
            "reg",
            &[self.mmio_range().start, self.mmio_range().len as u64],
        )?;
        if let Some(interrupts) = self.interrupts() {
            fdt.property_array_u32("interrupts", interrupts)?;
        }

        fdt.end_node(node)?;

        Ok(())
    }
}

pub struct VirtIoMmioAdaptor<T>(T);

impl<T> From<T> for VirtIoMmioAdaptor<T>
where
    T: VirtIoMmio,
{
    fn from(value: T) -> Self {
        VirtIoMmioAdaptor(value)
    }
}

impl<T> AsRef<T> for VirtIoMmioAdaptor<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for VirtIoMmioAdaptor<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Device for VirtIoMmioAdaptor<T>
where
    T: VirtIoMmio + Subsystem,
{
    fn name(&self) -> String {
        T::NAME.to_string()
    }

    fn as_mmio_device(&self) -> Option<&dyn MmioDevice> {
        Some(self)
    }

    fn as_mmio_device_mut(&mut self) -> Option<&mut dyn MmioDevice> {
        Some(self)
    }
}

impl<T> MmioDevice for VirtIoMmioAdaptor<T>
where
    T: VirtIoMmio + Subsystem,
{
    fn mmio_range(&self) -> MmioRange {
        self.0.mmio_range()
    }

    fn mmio_read(&mut self, offset: u64, len: usize, data: &mut [u8]) {
        let offset: usize = offset.try_into().unwrap();
        if offset < DEVICE_SPECIFIC_CONFIGURATION_OFFSET {
            if let Some(reg) = ControlRegister::from_repr(offset as u16) {
                assert_eq!(len, 4);
                assert_eq!(data.len(), 4);

                let val = self.0.read_control_register(reg).unwrap();
                data.copy_from_slice(&val.to_le_bytes());
            } else {
                warn!(
                    device = self.name(),
                    offset,
                    len,
                    ?data,
                    "read from invalid offset of virtio-mmio device"
                );
            }
        } else if let Err(err) = self.0.read_device_configuration(
            offset - DEVICE_SPECIFIC_CONFIGURATION_OFFSET,
            len,
            data,
        ) {
            error!(
                name = self.name(),
                ?err,
                "Failed to read device configuration"
            );
        }
    }

    fn mmio_write(&mut self, offset: u64, len: usize, data: &[u8]) {
        let offset: usize = offset.try_into().unwrap();
        if offset < DEVICE_SPECIFIC_CONFIGURATION_OFFSET {
            if let Some(reg) = ControlRegister::from_repr(offset as u16) {
                assert_eq!(len, 4);
                assert_eq!(data.len(), 4);

                self.0
                    .write_control_register(reg, u32::from_le_bytes(data.try_into().unwrap()))
                    .unwrap();
            } else {
                warn!(
                    device = self.name(),
                    offset,
                    len,
                    ?data,
                    "write from invalid offset of virtio-mmio device"
                );
            }
        } else if let Err(err) = self.0.write_device_configuration(
            offset - DEVICE_SPECIFIC_CONFIGURATION_OFFSET,
            len,
            data,
        ) {
            error!(
                name = self.name(),
                ?err,
                "Failed to write device configuration"
            );
        }
    }

    fn generate_dt(&self, fdt: &mut vm_fdt::FdtWriter) -> Result<(), vm_fdt::Error> {
        self.0.generate_dt(fdt)
    }
}
