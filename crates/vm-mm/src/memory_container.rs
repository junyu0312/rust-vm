use crate::allocator::AllocatorKind;

pub trait MemoryContainer: Send + Sync + 'static {
    fn kind(&self) -> AllocatorKind;

    fn align(&self) -> Option<usize>;

    fn hva(&self) -> *mut u8;

    fn length(&self) -> usize;

    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.hva(), self.length()) }
    }

    fn copy_from_slice(&self, src: &[u8]) {
        let point = unsafe { std::slice::from_raw_parts_mut(self.hva(), self.length()) };
        point.copy_from_slice(src);
    }
}
