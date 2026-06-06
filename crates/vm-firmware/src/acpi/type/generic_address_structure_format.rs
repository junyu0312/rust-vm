use zerocopy::Immutable;
use zerocopy::IntoBytes;

#[derive(Default, Immutable, IntoBytes)]
#[repr(C, packed)]
pub struct GenericAddressStructureFormat {}
