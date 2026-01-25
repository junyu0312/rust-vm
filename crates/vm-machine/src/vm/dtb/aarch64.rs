use vm_core::arch::Arch;
use vm_core::virt::Virt;
use vm_fdt::FdtWriter;

use crate::vm::Vm;

impl<V> Vm<V>
where
    V: Virt,
{
    pub fn generate_dtb(&self, cmdline: Option<&str>) -> anyhow::Result<Vec<u8>> {
        let mut fdt = FdtWriter::new()?;
        let root_node = fdt.begin_node("")?;

        fdt.property_string("compatible", "linux,virt")?;
        fdt.property_u32("#address-cells", 2)?;
        fdt.property_u32("#size-cells", 2)?;

        {
            let memory_node =
                fdt.begin_node(&format!("memory@{}", <V::Arch as Arch>::BASE_ADDRESS))?;
            fdt.property_string("device_type", "memory")?;
            fdt.property_array_u64(
                "reg",
                &[<V::Arch as Arch>::BASE_ADDRESS, self.memory_size as u64],
            )?;
            fdt.end_node(memory_node)?;
        }

        {
            let cpu_node = fdt.begin_node("cpus")?;
            fdt.property_u32("#address-cells", 1)?;
            fdt.property_u32("#size-cells", 0)?;
            for (i, _vcpu) in self.virt.get_vcpus().iter().enumerate() {
                let cpu_node = fdt.begin_node(&format!("cpu@{}", i))?;
                fdt.property_string("device_type", "cpu")?;
                fdt.property_string("compatible", "arm,cortex-a72")?;
                fdt.property_u32("reg", i as u32)?;
                fdt.end_node(cpu_node)?;
            }
            fdt.end_node(cpu_node)?;
        }

        {
            let serial_node = fdt.begin_node("uart@09000000")?;
            fdt.property_string("compatible", "ns16550a")?;
            fdt.property_array_u64("reg", &[0x09000000, 0x1000])?;
            fdt.property_u32("clock-frequency", 24000000)?;
            fdt.property_u32("current-speed", 115200)?;
            fdt.end_node(serial_node)?;
        }

        // {
        //     let gic_node = fdt.begin_node("interrupt-controller@8000000")?;
        //     fdt.property_string("compatible", "arm,gic-v3")?;
        //     fdt.property_null("interrupt-controller")?;
        //     fdt.property_u32("#interrupt-cells", 3)?;
        //     fdt.property_array_u64(
        //         "reg",
        //         &[
        //             0x08000000, 0x10000, // GICD
        //             0x080a0000, 0x200000, // GICR
        //         ],
        //     )?;
        //     fdt.property_u32("#address-cells", 2)?;
        //     fdt.property_u32("#size-cells", 2)?;
        //     fdt.end_node(gic_node)?;
        // }

        {
            let timer_node = fdt.begin_node("timer")?;
            fdt.property_string("compatible", "arm,armv8-timer")?;
            // fdt.property_string("interrupt-parent", "interrupt-controller@8000000")?;
            // GIC PPI interrupts
            // <type irq flags>
            // type: 1 = PPI
            // flags: 4 = IRQ_TYPE_LEVEL_HIGH
            fdt.property_array_u32(
                "interrupts",
                &[
                    1, 13, 4, // CNTPNS (EL1 physical timer)
                    1, 14, 4, // CNTPS  (secure timer)
                    1, 11, 4, 1, 10, 4, // CNTV   (virtual timer)
                ],
            )?;
            fdt.end_node(timer_node)?;
        }

        {
            let chosen_node = fdt.begin_node("chosen")?;
            if let Some(cmdline) = cmdline {
                fdt.property_string("bootargs", cmdline)?;
            }
            fdt.end_node(chosen_node)?;
        }

        fdt.end_node(root_node)?;
        let dtb = fdt.finish()?;

        anyhow::Ok(dtb)
    }
}
