use applevisor_sys::hv_exit_reason_t;
use tracing::trace;

use crate::device::pio::IoAddressSpace;
use crate::vcpu::Vcpu;
use crate::vcpu::arch::aarch64::AArch64Vcpu;

pub struct HvpVcpu {
    vcpu_id: u64,
    vcpu: applevisor::vcpu::Vcpu,
}

impl HvpVcpu {
    pub fn new(vcpu_id: u64, vcpu: applevisor::vcpu::Vcpu) -> Self {
        HvpVcpu { vcpu_id, vcpu }
    }
}

impl Vcpu for HvpVcpu {
    fn run(&mut self, _device: &mut IoAddressSpace) -> anyhow::Result<()> {
        loop {
            self.vcpu.run()?;

            let exit_info = self.vcpu.get_exit_info();

            trace!(self.vcpu_id, ?exit_info, "vm exit");

            match exit_info.reason {
                hv_exit_reason_t::CANCELED => todo!(),
                hv_exit_reason_t::EXCEPTION => todo!(),
                hv_exit_reason_t::VTIMER_ACTIVATED => todo!(),
                hv_exit_reason_t::UNKNOWN => todo!(),
            }
        }
    }
}

impl AArch64Vcpu for HvpVcpu {}
