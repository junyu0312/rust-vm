use strum_macros::FromRepr;
use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;

#[derive(FromRepr)]
pub enum VirtioEntropyVirtqueue {
    Requestq = 0,
}

#[derive(Default, FromBytes, IntoBytes, Immutable)]
#[repr(C, packed)]
pub struct VirtioEntropyConfig {}
