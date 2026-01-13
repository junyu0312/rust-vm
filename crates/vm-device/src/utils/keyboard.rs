use std::collections::HashMap;

use lazy_static::lazy_static;
use maplit::hashmap;

lazy_static! {
    pub static ref SCANCODE_SET2_MAP: HashMap<u8, Vec<u8>> = hashmap! {
        b'c' => vec![0x21, 0xf0, 0x21]
    };
}
