use zerocopy::{Immutable, IntoBytes};

#[derive(Default, Immutable, IntoBytes)]
#[repr(C, packed)]
pub struct GenericAddressStructureFormat {}
