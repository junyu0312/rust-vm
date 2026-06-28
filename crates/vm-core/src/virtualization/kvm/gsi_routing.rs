use std::sync::LazyLock;
use std::sync::Mutex;

use kvm_bindings::KvmIrqRouting;
use kvm_bindings::kvm_irq_routing_entry;
use kvm_bindings::kvm_irq_routing_entry__bindgen_ty_1;
use kvm_bindings::kvm_irq_routing_irqchip;
use kvm_bindings::kvm_irq_routing_msi;
use kvm_bindings::kvm_irq_routing_msi__bindgen_ty_1;

#[cfg(target_arch = "x86_64")]
use crate::virtualization::kvm::gsi_routing::x86_64::new_irq_gsi_routing;

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "x86_64")]
mod x86_64;

pub enum KvmSgiRoutingEntryU {
    Irqchip {
        irqchip: u32,
        pin: u32,
    },
    Msi {
        address_lo: u32,
        address_hi: u32,
        data: u32,
    },
}

impl From<&KvmSgiRoutingEntryU> for kvm_irq_routing_entry__bindgen_ty_1 {
    fn from(u: &KvmSgiRoutingEntryU) -> Self {
        match u {
            KvmSgiRoutingEntryU::Irqchip { irqchip, pin } => kvm_irq_routing_entry__bindgen_ty_1 {
                irqchip: kvm_irq_routing_irqchip {
                    irqchip: *irqchip,
                    pin: *pin,
                },
            },
            KvmSgiRoutingEntryU::Msi {
                address_lo,
                address_hi,
                data,
            } => kvm_irq_routing_entry__bindgen_ty_1 {
                msi: kvm_irq_routing_msi {
                    address_lo: *address_lo,
                    address_hi: *address_hi,
                    data: *data,
                    __bindgen_anon_1: kvm_irq_routing_msi__bindgen_ty_1 { pad: 0 },
                },
            },
        }
    }
}

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum KvmSgiRoutingEntryType {
    Irqchip = 1,
    Msi = 2,
}

pub struct KvmSgiRoutingEntry {
    pub gsi: u32,
    pub r#type: KvmSgiRoutingEntryType,
    pub u: KvmSgiRoutingEntryU,
}

impl From<&KvmSgiRoutingEntry> for kvm_irq_routing_entry {
    fn from(entry: &KvmSgiRoutingEntry) -> Self {
        kvm_irq_routing_entry {
            gsi: entry.gsi,
            type_: entry.r#type as u32,
            flags: 0,
            pad: 0,
            u: (&entry.u).into(),
        }
    }
}

#[derive(Default)]
pub struct KvmGsiRouting {
    nr: u32,
    entries: Vec<KvmSgiRoutingEntry>,
}

impl KvmGsiRouting {
    pub fn push(&mut self, entry: KvmSgiRoutingEntry) {
        self.entries.push(entry);
    }

    pub fn add_intx_gsi_routing(&mut self, gsi: u32, irqchip: u32, pin: u32) {
        self.entries.push(KvmSgiRoutingEntry {
            gsi,
            r#type: KvmSgiRoutingEntryType::Irqchip,
            u: KvmSgiRoutingEntryU::Irqchip { irqchip, pin },
        });
    }

    pub fn add_msi_gsi_routing(&mut self, gsi: u32, address_lo: u32, address_hi: u32, data: u32) {
        self.entries.push(KvmSgiRoutingEntry {
            gsi,
            r#type: KvmSgiRoutingEntryType::Msi,
            u: KvmSgiRoutingEntryU::Msi {
                address_lo,
                address_hi,
                data,
            },
        });
    }
}

impl TryFrom<&KvmGsiRouting> for KvmIrqRouting {
    type Error = vmm_sys_util::fam::Error;

    fn try_from(routing: &KvmGsiRouting) -> Result<Self, Self::Error> {
        let mut kvm_irq_routing = KvmIrqRouting::new(routing.nr as usize)?;

        for entry in &routing.entries {
            kvm_irq_routing.push(entry.into())?;
        }

        Ok(kvm_irq_routing)
    }
}

static KVM_SGI_ROUTING: LazyLock<Mutex<KvmGsiRouting>> =
    LazyLock::new(|| Mutex::new(new_irq_gsi_routing()));

pub fn get_kvm_sgi_routing_instance() -> &'static Mutex<KvmGsiRouting> {
    &KVM_SGI_ROUTING
}
