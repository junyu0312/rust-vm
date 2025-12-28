use crate::boot::ArchBootParams;
use crate::boot::BootHeaderEntry;
use crate::boot::VariantSizeValue;

macro_rules! define_real_mode_kernel_header {
    ($name:ident, $offset:expr, $size:expr, $default:expr) => {
        pub const $name: BootHeaderEntry = BootHeaderEntry {
            offset: $offset,
            size: $size,
            value: $default,
        };
    };
}

define_real_mode_kernel_header!(SETUP_SECTS, 0x1F1, 1, VariantSizeValue::U8(0));
define_real_mode_kernel_header!(ROOT_FLAGS, 0x1F2, 2, VariantSizeValue::U16(0));
define_real_mode_kernel_header!(BOOT_FLAG, 0x1FE, 2, VariantSizeValue::U16(0xAA55));
define_real_mode_kernel_header!(JUMP, 0x200, 2, VariantSizeValue::U16(0));
define_real_mode_kernel_header!(HEADER, 0x202, 4, VariantSizeValue::U32(0x53726448));
define_real_mode_kernel_header!(VERSION, 0x206, 2, VariantSizeValue::U16(0));
define_real_mode_kernel_header!(LOAD_FLAGS, 0x211, 1, VariantSizeValue::U8(0));
define_real_mode_kernel_header!(RAMDISK_IMAGE, 0x218, 4, VariantSizeValue::U32(0));
define_real_mode_kernel_header!(RAMDISK_SIZE, 0x21C, 4, VariantSizeValue::U32(0));
define_real_mode_kernel_header!(HEAD_END_PTR, 0x224, 2, VariantSizeValue::U16(0));
define_real_mode_kernel_header!(CMD_LINE_PTR, 0x228, 4, VariantSizeValue::U32(0));
define_real_mode_kernel_header!(CMDLINE_SIZE, 0x238, 4, VariantSizeValue::U32(0));

pub struct X86BootParams<'a> {
    mem: &'a mut [u8],
}

impl<'a> ArchBootParams for X86BootParams<'a> {
    fn as_mut_slice(&mut self) -> &mut [u8] {
        self.mem
    }

    fn reset(&mut self) {
        self.write(&SETUP_SECTS, SETUP_SECTS.value);
    }
}
