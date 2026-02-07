use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

use crate::pci::types::configuration_space::common::HeaderCommon;

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C, packed)]
pub struct Type1Header {
    common: HeaderCommon,
    bar0: u32,
    bar1: u32,
}
