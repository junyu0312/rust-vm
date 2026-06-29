use strum_macros::FromRepr;
use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

use crate::device::capability::PciCapId;
use crate::types::configuration_space::capability::StandardCapability;

pub const PCI_MSI_FLAGS_ENABLE: u16 = 0x0001; /* MSI feature enabled */
pub const PCI_MSI_FLAGS_QMASK: u16 = 0x000e; /* Maximum queue size available */
pub const PCI_MSI_FLAGS_QSIZE: u16 = 0x0070; /* Message queue size configured */
pub const PCI_MSI_FLAGS_64BIT: u16 = 0x0080; /* 64-bit addresses allowed */
pub const PCI_MSI_FLAGS_MASKBIT: u16 = 0x0100; /* Per-vector masking capable */

#[derive(Clone, Copy, FromRepr)]
#[repr(u8)]
pub enum PciMsiMmc {
    N1 = 0b000,
    N2 = 0b001,
    N4 = 0b010,
    N8 = 0b011,
    N16 = 0b100,
    N32 = 0b101,
}

impl PciMsiMmc {
    pub fn vectors(&self) -> u8 {
        1 << *self as u8
    }
}

pub trait PciMsiCapOps {
    fn mmc(&self) -> usize {
        1 << (((self.ctrl() & PCI_MSI_FLAGS_QMASK) >> 1) as usize)
    }

    fn mme(&self) -> usize {
        1 << (((self.ctrl() & PCI_MSI_FLAGS_QSIZE) >> 4) as usize)
    }

    fn ctrl(&self) -> u16;

    fn set_ctrl(&mut self, ctrl: u16);

    fn address_lo(&self) -> u32;

    fn address_hi(&self) -> u32;

    fn data(&self) -> u16;

    fn vector_data(&self, vector: usize) -> u32 {
        let mut data = self.data() as u32;
        data &= !(self.mme() as u32 - 1);
        data |= vector as u32;
        data
    }
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C, packed)]
pub struct PciMsiCap {
    control: u16,
    address_lo: u32,
    data: u16,
}

impl PciMsiCapOps for PciMsiCap {
    fn ctrl(&self) -> u16 {
        self.control
    }

    fn set_ctrl(&mut self, ctrl: u16) {
        self.control = ctrl;
    }

    fn address_lo(&self) -> u32 {
        self.address_lo
    }

    fn address_hi(&self) -> u32 {
        0
    }

    fn data(&self) -> u16 {
        self.data
    }
}

impl PciMsiCap {
    pub fn new(mmc: PciMsiMmc) -> Self {
        let control = (mmc as u16) << 1;

        PciMsiCap {
            control,
            address_lo: Default::default(),
            data: Default::default(),
        }
    }
}

impl From<PciMsiCap> for StandardCapability {
    fn from(cap: PciMsiCap) -> Self {
        StandardCapability::new(PciCapId::Msi as u8, cap.as_bytes().to_vec())
    }
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C, packed)]
pub struct PciMsiCap64 {
    control: u16,
    address_lo: u32,
    address_hi: u32,
    data: u16,
}

impl PciMsiCapOps for PciMsiCap64 {
    fn ctrl(&self) -> u16 {
        self.control
    }

    fn set_ctrl(&mut self, ctrl: u16) {
        self.control = ctrl;
    }

    fn address_lo(&self) -> u32 {
        self.address_lo
    }

    fn address_hi(&self) -> u32 {
        self.address_hi
    }

    fn data(&self) -> u16 {
        self.data
    }
}

impl PciMsiCap64 {
    pub fn new(mmc: PciMsiMmc) -> Self {
        let control = (mmc as u16) << 1 | PCI_MSI_FLAGS_64BIT;

        PciMsiCap64 {
            control,
            address_lo: Default::default(),
            address_hi: Default::default(),
            data: Default::default(),
        }
    }
}

impl From<PciMsiCap64> for StandardCapability {
    fn from(cap: PciMsiCap64) -> Self {
        StandardCapability::new(PciCapId::Msi as u8, cap.as_bytes().to_vec())
    }
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C, packed)]
pub struct PciMsiCapMask {
    control: u16,
    address_lo: u32,
    data: u16,
    reserved: u16,
    mask_bits: u32,
    pending_bits: u32,
}

impl PciMsiCapOps for PciMsiCapMask {
    fn ctrl(&self) -> u16 {
        self.control
    }

    fn set_ctrl(&mut self, ctrl: u16) {
        self.control = ctrl;
    }

    fn address_lo(&self) -> u32 {
        self.address_lo
    }

    fn address_hi(&self) -> u32 {
        0
    }

    fn data(&self) -> u16 {
        self.data
    }
}

impl PciMsiCapMask {
    pub fn new(mmc: PciMsiMmc) -> Self {
        let control = (mmc as u16) << 1 | PCI_MSI_FLAGS_MASKBIT;

        PciMsiCapMask {
            control,
            address_lo: Default::default(),
            data: Default::default(),
            reserved: Default::default(),
            mask_bits: Default::default(),
            pending_bits: Default::default(),
        }
    }
}

impl From<PciMsiCapMask> for StandardCapability {
    fn from(cap: PciMsiCapMask) -> Self {
        StandardCapability::new(PciCapId::Msi as u8, cap.as_bytes().to_vec())
    }
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C, packed)]
pub struct PciMsiCap64Mask {
    control: u16,
    address_lo: u32,
    address_hi: u32,
    data: u16,
    reserved: u16,
    mask_bits: u32,
    pending_bits: u32,
}

impl PciMsiCapOps for PciMsiCap64Mask {
    fn ctrl(&self) -> u16 {
        self.control
    }

    fn set_ctrl(&mut self, ctrl: u16) {
        self.control = ctrl;
    }

    fn address_lo(&self) -> u32 {
        self.address_lo
    }

    fn address_hi(&self) -> u32 {
        self.address_hi
    }

    fn data(&self) -> u16 {
        self.data
    }
}

impl PciMsiCap64Mask {
    pub fn new(mmc: PciMsiMmc) -> Self {
        let control = (mmc as u16) << 1 | PCI_MSI_FLAGS_64BIT | PCI_MSI_FLAGS_MASKBIT;

        PciMsiCap64Mask {
            control,
            address_lo: Default::default(),
            address_hi: Default::default(),
            data: Default::default(),
            reserved: Default::default(),
            mask_bits: Default::default(),
            pending_bits: Default::default(),
        }
    }
}

impl From<PciMsiCap64Mask> for StandardCapability {
    fn from(cap: PciMsiCap64Mask) -> Self {
        StandardCapability::new(PciCapId::Msi as u8, cap.as_bytes().to_vec())
    }
}
