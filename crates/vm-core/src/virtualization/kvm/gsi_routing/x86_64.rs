use crate::virtualization::kvm::gsi_routing::KvmGsiRouting;

const IRQCHIP_MASTER: u32 = 0;
const IRQCHIP_SLAVE: u32 = 1;
const IRQCHIP_IOAPIC: u32 = 2;

pub fn new_irq_gsi_routing() -> KvmGsiRouting {
    let mut gsi_routing = KvmGsiRouting::default();

    for i in 0..8 {
        if i != 2 {
            gsi_routing.add_intx_gsi_routing(i, IRQCHIP_MASTER, i);
        }
    }

    for i in 8..16 {
        gsi_routing.add_intx_gsi_routing(i, IRQCHIP_SLAVE, i - 8);
    }

    for i in 0..24 {
        if i == 0 {
            gsi_routing.add_intx_gsi_routing(i, IRQCHIP_IOAPIC, 2);
        } else if i != 2 {
            gsi_routing.add_intx_gsi_routing(i, IRQCHIP_IOAPIC, i);
        }
    }

    gsi_routing
}
