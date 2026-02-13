use crate::types::interrupt::InterruptMapEntryArch;

pub struct InterruptMapEntryX86_64 {}

impl InterruptMapEntryArch for InterruptMapEntryX86_64 {
    fn to_vec(&self) -> Vec<u32> {
        todo!()
    }
}
