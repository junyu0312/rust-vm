use kvm_bindings::*;

use crate::vcpu::arch::aarch64::reg::Reg;

pub struct RegisterEncoder;

impl RegisterEncoder {
    pub fn encode_core_register(id: u16) -> u64 {
        KVM_REG_ARM64 | KVM_REG_SIZE_U64 | (KVM_REG_ARM_CORE as u64) | (id as u64)
    }
}

impl Reg {
    pub fn to_kvm_reg(&self) -> u64 {
        match self {
            Reg::X0 => RegisterEncoder::encode_core_register(0),
            Reg::X1 => RegisterEncoder::encode_core_register(1),
            Reg::X2 => RegisterEncoder::encode_core_register(2),
            Reg::X3 => RegisterEncoder::encode_core_register(3),
            Reg::X4 => RegisterEncoder::encode_core_register(4),
            Reg::X5 => RegisterEncoder::encode_core_register(5),
            Reg::X6 => RegisterEncoder::encode_core_register(6),
            Reg::X7 => RegisterEncoder::encode_core_register(7),
            Reg::X8 => RegisterEncoder::encode_core_register(8),
            Reg::X9 => RegisterEncoder::encode_core_register(9),
            Reg::X10 => RegisterEncoder::encode_core_register(10),
            Reg::X11 => RegisterEncoder::encode_core_register(11),
            Reg::X12 => RegisterEncoder::encode_core_register(12),
            Reg::X13 => RegisterEncoder::encode_core_register(13),
            Reg::X14 => RegisterEncoder::encode_core_register(14),
            Reg::X15 => RegisterEncoder::encode_core_register(15),
            Reg::X16 => RegisterEncoder::encode_core_register(16),
            Reg::X17 => RegisterEncoder::encode_core_register(17),
            Reg::X18 => RegisterEncoder::encode_core_register(18),
            Reg::X19 => RegisterEncoder::encode_core_register(19),
            Reg::X20 => RegisterEncoder::encode_core_register(20),
            Reg::X21 => RegisterEncoder::encode_core_register(21),
            Reg::X22 => RegisterEncoder::encode_core_register(22),
            Reg::X23 => RegisterEncoder::encode_core_register(23),
            Reg::X24 => RegisterEncoder::encode_core_register(24),
            Reg::X25 => RegisterEncoder::encode_core_register(25),
            Reg::X26 => RegisterEncoder::encode_core_register(26),
            Reg::X27 => RegisterEncoder::encode_core_register(27),
            Reg::X28 => RegisterEncoder::encode_core_register(28),
            Reg::X29 => RegisterEncoder::encode_core_register(29),
            Reg::X30 => RegisterEncoder::encode_core_register(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_core_register() -> anyhow::Result<()> {
        assert_eq!(Reg::X0.to_kvm_reg(), 0x6030_0000_0010_0000);

        Ok(())
    }
}
