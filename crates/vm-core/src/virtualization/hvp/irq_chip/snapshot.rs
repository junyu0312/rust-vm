use std::collections::BTreeMap;

use applevisor_sys::hv_error_t;
use applevisor_sys::hv_gic_get_distributor_reg;
use applevisor_sys::hv_gic_get_msi_reg;
use applevisor_sys::hv_gic_set_distributor_reg;
use applevisor_sys::hv_gic_set_msi_reg;
use applevisor_sys::hv_gic_set_state;
use applevisor_sys::hv_gic_state_create;
use applevisor_sys::hv_gic_state_get_data;
use applevisor_sys::hv_gic_state_get_size;
use serde::Deserialize;
use serde::Serialize;
use strum::IntoEnumIterator;
use tracing::error;

use crate::arch::irq::error::IrqChipError;
use crate::virtualization::hvp::hv_unsafe_call;
use crate::virtualization::hvp::irq_chip::HvpGicV3;
use crate::virtualization::hvp::irq_chip::snapshot::reigster::DistributorRegister;
use crate::virtualization::hvp::irq_chip::snapshot::reigster::MsiRegister;

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
mod reigster;

#[derive(Serialize, Deserialize)]
pub struct HvpGicV3Snapshot {
    distributor: BTreeMap<DistributorRegister, u64>,
    msi: BTreeMap<MsiRegister, u64>,
    state: Vec<u8>,
}

impl HvpGicV3 {
    pub fn build_snapshot(&self) -> Result<HvpGicV3Snapshot, IrqChipError> {
        let distributor = {
            let mut distributor_regs = BTreeMap::new();

            for reg in DistributorRegister::iter() {
                let mut value = 0;
                hv_unsafe_call!(hv_gic_get_distributor_reg(reg.into(), &mut value))
                    .map_err(|_| IrqChipError::SaveSnapshot)?;
                distributor_regs.insert(reg, value);
            }

            distributor_regs
        };

        let msi = {
            let mut msi_regs = BTreeMap::new();

            for reg in MsiRegister::iter() {
                let mut value = 0;
                hv_unsafe_call!(hv_gic_get_msi_reg(reg.into(), &mut value))
                    .map_err(|_| IrqChipError::SaveSnapshot)?;
                msi_regs.insert(reg, value);
            }

            msi_regs
        };

        let state = {
            let state = unsafe { hv_gic_state_create() };
            let mut gic_state_data = 0;
            hv_unsafe_call!(hv_gic_state_get_size(state, &mut gic_state_data))
                .map_err(|_| IrqChipError::SaveSnapshot)?;
            let mut buf = Vec::<u8>::with_capacity(gic_state_data);
            hv_unsafe_call!(hv_gic_state_get_data(state, buf.as_mut_ptr() as _))
                .map_err(|_| IrqChipError::SaveSnapshot)?;
            unsafe { buf.set_len(gic_state_data) };

            buf
        };

        let snap = HvpGicV3Snapshot {
            distributor,
            msi,
            state,
        };

        Ok(snap)
    }

    pub fn install_snapshot(&mut self, snap: HvpGicV3Snapshot) -> Result<(), IrqChipError> {
        for (reg, value) in snap.distributor {
            hv_unsafe_call!(hv_gic_set_distributor_reg(reg.into(), value)).map_err(|err| {
                error!(?reg, ?err);
                IrqChipError::LoadSnapshot(Box::new(err))
            })?;
        }

        for (reg, value) in snap.msi {
            hv_unsafe_call!(hv_gic_set_msi_reg(reg.into(), value)).map_err(|err| {
                error!(?reg, ?err);
                IrqChipError::LoadSnapshot(Box::new(err))
            })?;
        }

        hv_unsafe_call!(hv_gic_set_state(snap.state.as_ptr() as _, snap.state.len())).map_err(
            |err| {
                error!(?err);
                IrqChipError::LoadSnapshot(Box::new(err))
            },
        )?;

        Ok(())
    }
}
