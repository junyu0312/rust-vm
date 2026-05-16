use crate::allocator::AllocatorKind;

pub trait MemoryContainer: Send + Sync + 'static {
    fn kind(&self) -> AllocatorKind;

    fn align(&self) -> Option<usize>;

    fn hva(&self) -> *mut u8;

    fn length(&self) -> usize;
}
