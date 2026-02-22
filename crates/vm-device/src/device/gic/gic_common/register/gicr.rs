use bitflags::bitflags;
use strum_macros::FromRepr;
use tracing::warn;

use crate::device::gic::gic_common::config::GicConfig;

const GIC_V3_REV: u32 = 0x3;
const REDIST_FRAME: usize = 64 * 1024;

bitflags! {
    #[derive(Default)]
    pub struct Ctlr: u32 {
        const UWP = 1 << 31;
        const DPG1S = 1 << 26;
        const DPG1NS = 1 << 25;
        const DPG0 = 1 << 24;
        const RWP = 1 << 3;
        const IR = 1 << 2;
        const CES = 1 << 1;
        const EnableLPIs = 1 << 0;
    }
}

pub struct Redistributor {
    pub ctlr: Ctlr,
    pub typer: u64,
    pub pidr2: u32,
}

impl Redistributor {
    pub fn new(cfg: &GicConfig, cpu: usize) -> Self {
        let ctlr = Ctlr::default();

        let typer = {
            let affinity_value = 0;
            let ppi_num = 0;
            let vsgi = 0;
            let common_lpiaff = 0;
            let processor_number = 0;
            let rvpeid = 0;
            let mpam = 0;
            let dpgs = 0;
            let last = (cpu == cfg.cpu_number - 1) as u64;
            let direct_lpi = 0;
            let dirty = 0; // When GICR_TYPER.VLPIS == 0, this field is RES0.
            let vlpis = cfg.vlpis as u64;
            let plpis = 0;

            (affinity_value << 32)
                | (ppi_num << 27)
                | (vsgi << 26)
                | (common_lpiaff << 24)
                | (processor_number << 8)
                | (rvpeid << 7)
                | (mpam << 6)
                | (dpgs << 5)
                | (last << 4)
                | (direct_lpi << 3)
                | (dirty << 2)
                | (vlpis << 1)
                | plpis
        };
        let pidr2 = GIC_V3_REV << 4;

        Redistributor { ctlr, typer, pidr2 }
    }

    pub fn mmio_read(&mut self, offset: u64, data: &mut [u8]) {
        match GICR::from_repr(offset.try_into().unwrap()) {
            Some(reg) => match reg {
                GICR::CTLR => data.copy_from_slice(&self.ctlr.bits().to_le_bytes()),
                GICR::IIDR => todo!(),
                GICR::TYPER => data.copy_from_slice(&self.typer.to_le_bytes()),
                GICR::STATUSR => todo!(),
                GICR::WAKER => todo!(),
                GICR::SETLPIR => todo!(),
                GICR::CLRLPIR => todo!(),
                GICR::PROPBASER => todo!(),
                GICR::PENDBASER => todo!(),
                GICR::INVLPIR => todo!(),
                GICR::INVALLR => todo!(),
                GICR::SYNCR => todo!(),
                GICR::IDREGS => todo!(),
                GICR::PIDR2 => data.copy_from_slice(&self.pidr2.to_le_bytes()),
            },
            None => todo!(),
        }
    }

    pub fn mmio_write(&mut self, offset: u64, _data: &[u8]) {
        match GICR::from_repr(offset.try_into().unwrap()) {
            Some(reg) => match reg {
                GICR::CTLR => todo!(),
                GICR::IIDR => todo!(),
                GICR::STATUSR => todo!(),
                GICR::WAKER => todo!(),
                GICR::SETLPIR => todo!(),
                GICR::CLRLPIR => todo!(),
                GICR::PROPBASER => todo!(),
                GICR::PENDBASER => todo!(),
                GICR::INVLPIR => todo!(),
                GICR::INVALLR => todo!(),
                GICR::SYNCR => todo!(),
                GICR::IDREGS => todo!(),
                _ => {
                    warn!(?reg, "writing to a RO register is ignored");
                }
            },
            None => todo!(),
        }
    }
}

#[derive(Debug, FromRepr)]
#[repr(u16)]
pub enum GICR {
    CTLR = 0x0000,
    IIDR = 0x0004,
    TYPER = 0x0008,
    STATUSR = 0x0010,
    WAKER = 0x0014,
    SETLPIR = 0x0040,
    CLRLPIR = 0x0048,
    PROPBASER = 0x0070,
    PENDBASER = 0x0078,
    INVLPIR = 0x00A0,
    INVALLR = 0x00B0,
    SYNCR = 0x00C0,
    IDREGS = 0xFFD0,
    PIDR2 = 0xFFE8,
}

pub struct RedistributorRegion {
    pub redist_stride: usize,
    pub redistributors: Vec<Redistributor>,
}

impl RedistributorRegion {
    pub fn new(cfg: &GicConfig) -> Self {
        let redist_stride = if let Some(redist_stride) = cfg.redist_stride {
            redist_stride
        } else if cfg.vlpis {
            REDIST_FRAME * 4 /* Skip VLPI_base + reserved page */
        } else {
            REDIST_FRAME * 2 /* Skip RD_base + SGI_base */
        };

        let mut redistributors = Vec::with_capacity(cfg.cpu_number);
        for cpu in 0..cfg.cpu_number {
            redistributors.push(Redistributor::new(cfg, cpu));
        }

        RedistributorRegion {
            redist_stride,
            redistributors,
        }
    }
}
