use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

use kvm_bindings::KvmIrqRouting;
use kvm_bindings::kvm_irq_routing_entry;
use kvm_bindings::kvm_irq_routing_entry__bindgen_ty_1;
use kvm_bindings::kvm_irq_routing_irqchip;
use kvm_bindings::kvm_irq_routing_msi;
use kvm_bindings::kvm_irq_routing_msi__bindgen_ty_1;

#[cfg(target_arch = "aarch64")]
use crate::virtualization::kvm::gsi_routing::aarch64::new_irq_gsi_routing;
#[cfg(target_arch = "x86_64")]
use crate::virtualization::kvm::gsi_routing::x86_64::new_irq_gsi_routing;

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[derive(PartialEq)]
pub enum KvmGsiRoutingEntryU {
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

impl From<&KvmGsiRoutingEntryU> for kvm_irq_routing_entry__bindgen_ty_1 {
    fn from(u: &KvmGsiRoutingEntryU) -> Self {
        match u {
            KvmGsiRoutingEntryU::Irqchip { irqchip, pin } => kvm_irq_routing_entry__bindgen_ty_1 {
                irqchip: kvm_irq_routing_irqchip {
                    irqchip: *irqchip,
                    pin: *pin,
                },
            },
            KvmGsiRoutingEntryU::Msi {
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

#[derive(Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum KvmGsiRoutingEntryType {
    Irqchip = 1,
    Msi = 2,
}

#[derive(PartialEq)]
pub struct KvmGsiRoutingEntry {
    pub gsi: u32,
    pub r#type: KvmGsiRoutingEntryType,
    pub u: KvmGsiRoutingEntryU,
}

impl From<&KvmGsiRoutingEntry> for kvm_irq_routing_entry {
    fn from(entry: &KvmGsiRoutingEntry) -> Self {
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
    irqchip_entries: Vec<KvmGsiRoutingEntry>,
    msi_entryies: HashMap<u32, KvmGsiRoutingEntry>,
}

impl KvmGsiRouting {
    pub fn add_intx_gsi_routing(&mut self, gsi: u32, irqchip: u32, pin: u32) {
        self.irqchip_entries.push(KvmGsiRoutingEntry {
            gsi,
            r#type: KvmGsiRoutingEntryType::Irqchip,
            u: KvmGsiRoutingEntryU::Irqchip { irqchip, pin },
        });
    }

    pub fn insert_or_update_msi_gsi_routing(
        &mut self,
        gsi: u32,
        address_lo: u32,
        address_hi: u32,
        data: u32,
    ) -> bool {
        let new = KvmGsiRoutingEntry {
            gsi,
            r#type: KvmGsiRoutingEntryType::Msi,
            u: KvmGsiRoutingEntryU::Msi {
                address_lo,
                address_hi,
                data,
            },
        };

        let old = self.msi_entryies.get(&gsi);

        if old == Some(&new) {
            return false;
        }

        self.msi_entryies.insert(gsi, new);

        true
    }

    pub fn remove_msi_gsi_routing(&mut self, gsi: u32) -> bool {
        self.msi_entryies.remove(&gsi).is_some()
    }
}

impl TryFrom<&KvmGsiRouting> for KvmIrqRouting {
    type Error = vmm_sys_util::fam::Error;

    fn try_from(routing: &KvmGsiRouting) -> Result<Self, Self::Error> {
        let mut kvm_irq_routing = KvmIrqRouting::new(routing.nr as usize)?;

        for entry in &routing.irqchip_entries {
            kvm_irq_routing.push(entry.into())?;
        }
        for entry in routing.msi_entryies.values() {
            kvm_irq_routing.push(entry.into())?;
        }

        Ok(kvm_irq_routing)
    }
}

static KVM_GSI_ROUTING: LazyLock<Mutex<KvmGsiRouting>> =
    LazyLock::new(|| Mutex::new(new_irq_gsi_routing()));

pub fn get_kvm_gsi_routing_instance() -> &'static Mutex<KvmGsiRouting> {
    &KVM_GSI_ROUTING
}
