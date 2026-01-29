use strum_macros::FromRepr;
use vm_core::device::Device;
use vm_core::device::mmio::MmioDevice;
use vm_core::device::mmio::MmioRange;
use vm_fdt::FdtWriter;

use crate::virtio::device::Subsystem;

#[derive(FromRepr)]
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
    QueueNumMax = 0x034,

    /// Queue size - Write Only
    QueueNum = 0x038,

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

pub trait VirtIoMmio {
    type Subsystem: Subsystem;

    const NAME: &str;

    fn mmio_range(&self) -> &MmioRange;

    fn interrupts(&self) -> Option<&[u32]>;

    fn write_control_register(&mut self, reg: ControlRegister, len: usize, data: &[u8]) {
        assert_eq!(len, 4);
        assert_eq!(data.len(), 4);
        let _val = u32::from_le_bytes(data.try_into().unwrap());

        match reg {
            ControlRegister::MagicValue => unreachable!("RO"),
            ControlRegister::Version => unreachable!("RO"),
            ControlRegister::DeviceId => unreachable!("RO"),
            ControlRegister::VendorId => unreachable!("RO"),
            ControlRegister::DeviceFeatures => unreachable!("RO"),
            ControlRegister::DeviceFeaturesSel => todo!(),
            ControlRegister::DriverFeatures => todo!(),
            ControlRegister::DriverFeaturesSel => todo!(),
            ControlRegister::QueueSel => todo!(),
            ControlRegister::QueueNumMax => unreachable!("RO"),
            ControlRegister::QueueNum => todo!(),
            ControlRegister::QueueReady => todo!(),
            ControlRegister::QueueNotify => todo!(),
            ControlRegister::InterruptStatus => unreachable!("RO"),
            ControlRegister::InterruptAck => todo!(),
            ControlRegister::Status => todo!(),
            ControlRegister::QueueDescLow => todo!(),
            ControlRegister::QueueDescHigh => todo!(),
            ControlRegister::QueueAvailLow => todo!(),
            ControlRegister::QueueAvailHigh => todo!(),
            ControlRegister::QueueUsedLow => todo!(),
            ControlRegister::QueueUsedHigh => todo!(),
            ControlRegister::ShmSel => todo!(),
            ControlRegister::ShmLenLow => unreachable!("RO"),
            ControlRegister::ShmLenHigh => unreachable!("RO"),
            ControlRegister::ShmBaseLow => todo!(),
            ControlRegister::ShmBaseHigh => todo!(),
            ControlRegister::ConfigGeneration => unreachable!("RO"),
        }
    }

    fn read_control_register(&mut self, reg: ControlRegister, _len: usize, _data: &mut [u8]) {
        match reg {
            ControlRegister::MagicValue => todo!(),
            ControlRegister::Version => todo!(),
            ControlRegister::DeviceId => todo!(),
            ControlRegister::VendorId => todo!(),
            ControlRegister::DeviceFeatures => todo!(),
            ControlRegister::DeviceFeaturesSel => todo!(),
            ControlRegister::DriverFeatures => todo!(),
            ControlRegister::DriverFeaturesSel => todo!(),
            ControlRegister::QueueSel => todo!(),
            ControlRegister::QueueNumMax => todo!(),
            ControlRegister::QueueNum => todo!(),
            ControlRegister::QueueReady => todo!(),
            ControlRegister::QueueNotify => todo!(),
            ControlRegister::InterruptStatus => todo!(),
            ControlRegister::InterruptAck => todo!(),
            ControlRegister::Status => todo!(),
            ControlRegister::QueueDescLow => todo!(),
            ControlRegister::QueueDescHigh => todo!(),
            ControlRegister::QueueAvailLow => todo!(),
            ControlRegister::QueueAvailHigh => todo!(),
            ControlRegister::QueueUsedLow => todo!(),
            ControlRegister::QueueUsedHigh => todo!(),
            ControlRegister::ShmSel => todo!(),
            ControlRegister::ShmLenLow => todo!(),
            ControlRegister::ShmLenHigh => todo!(),
            ControlRegister::ShmBaseLow => todo!(),
            ControlRegister::ShmBaseHigh => todo!(),
            ControlRegister::ConfigGeneration => todo!(),
        }
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

impl<T> Device for VirtIoMmioAdaptor<T>
where
    T: VirtIoMmio,
{
    fn name(&self) -> &str {
        T::NAME
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
    T: VirtIoMmio,
{
    fn mmio_range(&self) -> &MmioRange {
        self.0.mmio_range()
    }

    fn mmio_read(&mut self, _offset: u64, _len: usize, _data: &mut [u8]) {
        todo!()
    }

    fn mmio_write(&mut self, _offset: u64, _len: usize, _data: &[u8]) {
        todo!()
    }

    fn generate_dt(&self, fdt: &mut vm_fdt::FdtWriter) -> Result<(), vm_fdt::Error> {
        self.0.generate_dt(fdt)
    }
}
