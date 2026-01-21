use kvm_bindings::*;

use crate::vcpu::arch::aarch64::reg::CoreRegister;

pub struct RegisterEncoder;

impl RegisterEncoder {
    pub fn encode_register(id: u16) -> u64 {
        KVM_REG_ARM64 | KVM_REG_SIZE_U64 | (KVM_REG_ARM_CORE as u64) | (id as u64)
    }
}

impl CoreRegister {
    pub fn to_kvm_reg(&self) -> u64 {
        /*
         * Refer to `https://docs.kernel.org/virt/kvm/api.html`
         */
        match self {
            CoreRegister::X0 => RegisterEncoder::encode_register(0x00),
            CoreRegister::X1 => RegisterEncoder::encode_register(0x02),
            CoreRegister::X2 => RegisterEncoder::encode_register(0x04),
            CoreRegister::X3 => RegisterEncoder::encode_register(0x06),
            CoreRegister::X4 => RegisterEncoder::encode_register(0x08),
            CoreRegister::X5 => RegisterEncoder::encode_register(0x0a),
            CoreRegister::X6 => RegisterEncoder::encode_register(0x0c),
            CoreRegister::X7 => RegisterEncoder::encode_register(0x0e),
            CoreRegister::X8 => RegisterEncoder::encode_register(0x10),
            CoreRegister::X9 => RegisterEncoder::encode_register(0x12),
            CoreRegister::X10 => RegisterEncoder::encode_register(0x14),
            CoreRegister::X11 => RegisterEncoder::encode_register(0x16),
            CoreRegister::X12 => RegisterEncoder::encode_register(0x18),
            CoreRegister::X13 => RegisterEncoder::encode_register(0x1a),
            CoreRegister::X14 => RegisterEncoder::encode_register(0x1c),
            CoreRegister::X15 => RegisterEncoder::encode_register(0x1e),
            CoreRegister::X16 => RegisterEncoder::encode_register(0x20),
            CoreRegister::X17 => RegisterEncoder::encode_register(0x22),
            CoreRegister::X18 => RegisterEncoder::encode_register(0x24),
            CoreRegister::X19 => RegisterEncoder::encode_register(0x26),
            CoreRegister::X20 => RegisterEncoder::encode_register(0x28),
            CoreRegister::X21 => RegisterEncoder::encode_register(0x2a),
            CoreRegister::X22 => RegisterEncoder::encode_register(0x2c),
            CoreRegister::X23 => RegisterEncoder::encode_register(0x2e),
            CoreRegister::X24 => RegisterEncoder::encode_register(0x30),
            CoreRegister::X25 => RegisterEncoder::encode_register(0x32),
            CoreRegister::X26 => RegisterEncoder::encode_register(0x34),
            CoreRegister::X27 => RegisterEncoder::encode_register(0x36),
            CoreRegister::X28 => RegisterEncoder::encode_register(0x38),
            CoreRegister::X29 => RegisterEncoder::encode_register(0x3a),
            CoreRegister::X30 => RegisterEncoder::encode_register(0x3c),
            CoreRegister::SP => RegisterEncoder::encode_register(0x3e),
            CoreRegister::PC => RegisterEncoder::encode_register(0x40),
            CoreRegister::PState => RegisterEncoder::encode_register(0x42),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_register() -> anyhow::Result<()> {
        assert_eq!(CoreRegister::X0.to_kvm_reg(), 0x6030_0000_0010_0000);

        Ok(())
    }
}
