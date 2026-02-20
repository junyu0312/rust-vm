use strum_macros::FromRepr;
use vm_core::device::Device;
use vm_core::device::mmio::MmioRange;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_core::device::mmio::mmio_device::MmioHandler;
use vm_core::irq::InterruptController;
use vm_core::irq::Phandle;

/*
 * Distributor registers. We assume we're running non-secure, with ARE
 * being set. Secure-only and non-ARE registers are not described.
 */
#[allow(non_camel_case_types)]
#[derive(FromRepr)]
#[repr(u16)]
enum GICD {
    CTLR = 0x0000,
    TYPER = 0x0004,
    IIDR = 0x0008,
    TYPER2 = 0x000C,
    STATUSR = 0x0010,
    SETSPI_NSR = 0x0040,
    CLRSPI_NSR = 0x0048,
    SETSPI_SR = 0x0050,
    CLRSPI_SR = 0x0058,
    IGROUPR = 0x0080,
    ISENABLER = 0x0100,
    ICENABLER = 0x0180,
    ISPENDR = 0x0200,
    ICPENDR = 0x0280,
    ISACTIVER = 0x0300,
    ICACTIVER = 0x0380,
    IPRIORITYR = 0x0400,
    ICFGR = 0x0C00,
    IGRPMODR = 0x0D00,
    NSACR = 0x0E00,
    IGROUPRnE = 0x1000,
    ISENABLERnE = 0x1200,
    ICENABLERnE = 0x1400,
    ISPENDRnE = 0x1600,
    ICPENDRnE = 0x1800,
    ISACTIVERnE = 0x1A00,
    ICACTIVERnE = 0x1C00,
    IPRIORITYRnE = 0x2000,
    ICFGRnE = 0x3000,
    IROUTER = 0x6000,
    IROUTERnE = 0x8000,
    IDREGS = 0xFFD0,
    PIDR2 = 0xFFE8,
}

pub struct GicV3DistributorHandler {
    distributor_base: u64,
    distributor_size: u64,
}

impl GicV3DistributorHandler {
    fn read_pidr2(&self, data: &mut [u8]) {
        todo!()
    }
}

impl MmioHandler for GicV3DistributorHandler {
    fn mmio_range(&self) -> MmioRange {
        MmioRange {
            start: self.distributor_base,
            len: self.distributor_size.try_into().unwrap(),
        }
    }

    fn mmio_read(&self, offset: u64, len: usize, data: &mut [u8]) {
        match GICD::from_repr(offset.try_into().unwrap()) {
            Some(gicd) => match gicd {
                GICD::CTLR => todo!(),
                GICD::TYPER => todo!(),
                GICD::IIDR => todo!(),
                GICD::TYPER2 => todo!(),
                GICD::STATUSR => todo!(),
                GICD::SETSPI_NSR => todo!(),
                GICD::CLRSPI_NSR => todo!(),
                GICD::SETSPI_SR => todo!(),
                GICD::CLRSPI_SR => todo!(),
                GICD::IGROUPR => todo!(),
                GICD::ISENABLER => todo!(),
                GICD::ICENABLER => todo!(),
                GICD::ISPENDR => todo!(),
                GICD::ICPENDR => todo!(),
                GICD::ISACTIVER => todo!(),
                GICD::ICACTIVER => todo!(),
                GICD::IPRIORITYR => todo!(),
                GICD::ICFGR => todo!(),
                GICD::IGRPMODR => todo!(),
                GICD::NSACR => todo!(),
                GICD::IGROUPRnE => todo!(),
                GICD::ISENABLERnE => todo!(),
                GICD::ICENABLERnE => todo!(),
                GICD::ISPENDRnE => todo!(),
                GICD::ICPENDRnE => todo!(),
                GICD::ISACTIVERnE => todo!(),
                GICD::ICACTIVERnE => todo!(),
                GICD::IPRIORITYRnE => todo!(),
                GICD::ICFGRnE => todo!(),
                GICD::IROUTER => todo!(),
                GICD::IROUTERnE => todo!(),
                GICD::IDREGS => todo!(),
                GICD::PIDR2 => todo!(),
            },
            None => todo!(),
        }
    }

    fn mmio_write(&self, offset: u64, len: usize, data: &[u8]) {
        todo!()
    }
}

pub struct GicV3RedistributorHandler {
    redistributor_base: u64,
    redistributor_size: u64,
}

impl MmioHandler for GicV3RedistributorHandler {
    fn mmio_range(&self) -> MmioRange {
        MmioRange {
            start: self.redistributor_base,
            len: self.redistributor_size.try_into().unwrap(),
        }
    }

    fn mmio_read(&self, offset: u64, len: usize, data: &mut [u8]) {
        todo!()
    }

    fn mmio_write(&self, offset: u64, len: usize, data: &[u8]) {
        todo!()
    }
}

pub struct GicV3Internal {
    distributor_base: u64,
    distributor_size: u64,
    redistributor_base: u64,
    redistributor_size: u64,
}

impl Device for GicV3Internal {
    fn name(&self) -> String {
        "gic-v3".to_string()
    }
}

impl MmioDevice for GicV3Internal {
    fn mmio_range_handlers(&self) -> Vec<Box<dyn MmioHandler>> {
        let distributor_handler = GicV3DistributorHandler {
            distributor_base: self.distributor_base,
            distributor_size: self.distributor_size,
        };

        let redistributor_handler = GicV3RedistributorHandler {
            redistributor_base: self.redistributor_base,
            redistributor_size: self.redistributor_size,
        };

        vec![
            Box::new(distributor_handler),
            Box::new(redistributor_handler),
        ]
    }

    fn generate_dt(&self, _fdt: &mut vm_fdt::FdtWriter) -> Result<(), vm_fdt::Error> {
        // skip
        Ok(())
    }
}

pub struct GicV3 {
    distributor_base: u64,
    distributor_size: u64,
    redistributor_base: u64,
    redistributor_size: u64,
}

impl GicV3 {
    pub fn new(distributor_base: u64, redistributor_base: u64) -> Self {
        GicV3 {
            distributor_base,
            distributor_size: 0x10000,
            redistributor_base,
            redistributor_size: 0x200000,
        }
    }

    pub fn get_device(&self) -> Box<GicV3Internal> {
        Box::new(GicV3Internal {
            distributor_base: self.distributor_base,
            distributor_size: self.distributor_size,
            redistributor_base: self.redistributor_base,
            redistributor_size: self.redistributor_size,
        })
    }
}

impl InterruptController for GicV3 {
    fn trigger_irq(&self, _irq_line: u32, _active: bool) {
        todo!()
    }

    fn send_msi(&self, _intid: u32) {
        todo!()
    }

    fn write_device_tree(
        &self,
        fdt: &mut vm_fdt::FdtWriter,
    ) -> anyhow::Result<vm_core::irq::Phandle> {
        let gic_node = fdt.begin_node(&format!(
            "interrupt-controller@{:016x}",
            self.distributor_base
        ))?;
        fdt.property_string("compatible", "arm,gic-v3")?;
        fdt.property_u32("#interrupt-cells", 3)?;
        fdt.property_null("interrupt-controller")?;
        fdt.property_u32("#address-cells", 2)?;
        fdt.property_u32("#size-cells", 2)?;
        fdt.property_phandle(Phandle::GIC as u32)?;
        fdt.property_array_u64(
            "reg",
            &[
                self.distributor_base,
                self.distributor_size,
                self.redistributor_base,
                self.redistributor_size,
            ],
        )?;
        fdt.property_null("ranges")?;

        fdt.end_node(gic_node)?;

        Ok(Phandle::GIC)
    }
}
