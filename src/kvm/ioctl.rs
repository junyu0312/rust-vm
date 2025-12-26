use nix::ioctl_none;

const KVM_IO: u8 = 0xAE;

ioctl_none!(kvm_create_vm, KVM_IO, 0x01);
