pub trait MemoryContainer: Send + Sync + 'static {
    fn hva(&self) -> *mut u8;

    fn length(&self) -> usize;
}
