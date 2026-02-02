#[repr(C)]
pub struct VirtioInputEvent {
    pub r#type: u16,
    pub code: u16,
    pub value: u32,
}
