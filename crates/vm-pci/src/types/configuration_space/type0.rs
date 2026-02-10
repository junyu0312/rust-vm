use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

use crate::types::configuration_space::common::HeaderCommon;

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C, packed)]
pub struct Type0Header {
    pub common: HeaderCommon,
    pub bar: [u32; 6],
}
