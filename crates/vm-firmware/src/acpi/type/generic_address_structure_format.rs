use zerocopy::Immutable;
use zerocopy::IntoBytes;

#[derive(Default, Immutable, IntoBytes)]
#[repr(C, packed)]
pub struct GenericAddressStructureFormat {
    address_space_id: u8,
    register_bit_width: u8,
    register_bit_offset: u8,
    access_size: u8,
    address: u64,
}
