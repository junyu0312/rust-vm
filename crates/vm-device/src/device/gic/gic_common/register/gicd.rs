use strum_macros::FromRepr;
use tracing::warn;

use crate::device::gic::gic_common::config::GicConfig;

const GIC_V3_REV: u32 = 0x3;

pub struct Distributor {
    pub typer: u32,
    pub iidr: u32,
    pub typer2: u32,
    pub pidr2: u32,
}

impl Distributor {
    pub fn new(config: &GicConfig) -> Self {
        let typer = {
            let espi_range = 0;
            let rss = 0;
            let no_1_n = 0;
            let a3v = 0;
            let id_bits = 10;
            let dvis = 0;
            let lpis = 0;
            let mbis = config.mbis as u32;
            let num_lpis = 0;
            let security_extn = if config.security_extn {
                unimplemented!("two security state is not implemented");
            } else {
                0
            };
            let nmi = config.nmi as u32;
            let espi = if config.extended_spi {
                unimplemented!("espi is not implemented");
            } else {
                0
            };
            let cpu_number = if config.are {
                0
            } else {
                config.cpu_number as u32 - 1
            };
            let itlines_number = 0b11111 << 0;

            (espi_range << 27)
                | (rss << 26)
                | (no_1_n << 25)
                | (a3v << 24)
                | (id_bits << 19)
                | (dvis << 18)
                | (lpis << 17)
                | (mbis << 16)
                | (num_lpis << 11)
                | (security_extn << 10)
                | (nmi << 9)
                | (espi << 8)
                | (cpu_number << 5)
                | itlines_number
        };

        let iidr = {
            let product_id = 0;
            let variant = 0;
            let revision = 0;
            let implementer = 0x43b;

            (product_id << 24) | (variant << 16) | (revision << 12) | implementer
        };

        /*
         * This register is present only when FEAT_GICv4p1 is implemented. Otherwise, direct accesses to
         * GICD_TYPER2 are RES0.
         */
        let typer2 = 0;

        let pidr2 = GIC_V3_REV << 4;

        Self {
            typer,
            iidr,
            typer2,
            pidr2,
        }
    }

    pub fn mmio_read(&mut self, offset: u64, data: &mut [u8]) {
        match GICD::from_repr(offset.try_into().unwrap()) {
            Some(reg) => match reg {
                GICD::CTLR => todo!(),
                GICD::TYPER => data.copy_from_slice(&self.typer.to_le_bytes()),
                GICD::IIDR => data.copy_from_slice(&self.iidr.to_le_bytes()),
                GICD::TYPER2 => data.copy_from_slice(&self.typer2.to_le_bytes()),
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
                GICD::PIDR2 => data.copy_from_slice(&self.pidr2.to_le_bytes()),
            },
            None => todo!(),
        }
    }

    pub fn mmio_write(&mut self, offset: u64, _data: &[u8]) {
        match GICD::from_repr(offset.try_into().unwrap()) {
            Some(reg) => match reg {
                GICD::CTLR => todo!(),
                GICD::IIDR => todo!(),
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
                _ => {
                    warn!(?reg, "writing to a RO register is ignored");
                }
            },
            None => todo!(),
        }
    }
}

/*
 * Distributor registers. We assume we're running non-secure, with ARE
 * being set. Secure-only and non-ARE registers are not described.
 */
#[allow(non_camel_case_types)]
#[derive(Debug, FromRepr)]
#[repr(u16)]
pub enum GICD {
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
