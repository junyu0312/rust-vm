use std::cell::OnceCell;

use thiserror::Error;
use vm_mm::manager::MemoryAddressSpace;
use zerocopy::FromBytes;
use zerocopy::FromZeros;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

const EDD_MBR_SIG_MAX: usize = 16; /* max number of signatures to store */
const EDDMAXNR: usize = 6;

const E820_MAX_ENTRIES_ZEROPAGE: usize = 128;

#[repr(C, packed)]
#[derive(FromZeros)]
struct ScreenInfo {
    orig_x: u8,             // 0x00
    orig_y: u8,             // 0x01
    ext_mem_k: u16,         // 0x02
    orig_video_page: u16,   // 0x04
    orig_video_mode: u8,    // 0x06
    orig_video_cols: u8,    // 0x07
    flags: u8,              // 0x08
    unused2: u8,            // 0x09
    orig_video_ega_bx: u16, // 0x0A
    unused3: u16,           // 0x0C
    orig_video_lines: u8,   // 0x0E
    orig_video_is_vga: u8,  // 0x0F
    orig_video_points: u16, // 0x10
    /* VESA graphic mode -- linear frame buffer */
    lfb_width: u16,       // 0x12
    lfb_height: u16,      // 0x14
    lfb_depth: u16,       // 0x16
    lfb_base: u32,        // 0x18
    lfb_size: u32,        // 0x1C
    cl_magic: u16,        // 0x20
    cl_offset: u16,       // 0x22
    lfb_linelength: u16,  // 0x24
    red_size: u8,         // 0x26
    red_pos: u8,          // 0x27
    green_size: u8,       // 0x28
    green_pos: u8,        // 0x29
    blue_size: u8,        // 0x2A
    blue_pos: u8,         // 0x2B
    rsvd_size: u8,        // 0x2C
    rsvd_pos: u8,         // 0x2D
    vesapm_seg: u16,      // 0x2E
    vesapm_off: u16,      // 0x30
    pages: u16,           // 0x32
    vesa_attributes: u16, // 0x34
    capabilities: u32,    // 0x36
    ext_lfb_base: u32,    // 0x3A
    _reserved: [u8; 2],   // 0x3E
}

#[repr(C, packed)]
#[derive(FromZeros)]
struct ApmBiosInfo {
    version: u16,
    cseg: u16,
    offset: u32,
    cseg_16: u16,
    dseg: u16,
    flags: u16,
    cseg_len: u16,
    cseg_16_len: u16,
    dseg_len: u16,
}

#[repr(C, packed)]
#[derive(FromZeros)]
struct IstInfo {
    signature: u32,
    command: u32,
    event: u32,
    perf_level: u32,
}

#[repr(C, packed)]
#[derive(FromZeros)]
struct SysDescTable {
    length: u16,
    table: [u8; 14],
}

#[repr(C, packed)]
#[derive(FromZeros)]
struct OlpcOfwHeader {
    ofw_magic: u32, /* OFW signature */
    ofw_version: u32,
    cif_handler: u32, /* callback into OFW */
    irq_desc_table: u32,
}

#[repr(C, packed)]
#[derive(FromZeros)]
struct EdidInfo {
    dummy: [u8; 128],
}

#[repr(C, packed)]
#[derive(FromZeros)]
struct EfiInfo {
    efi_loader_signature: u32,
    efi_systab: u32,
    efi_memdesc_size: u32,
    efi_memdesc_version: u32,
    efi_memmap: u32,
    efi_memmap_size: u32,
    efi_systab_hi: u32,
    efi_memmap_hi: u32,
}

#[repr(C, packed)]
#[derive(Debug, FromBytes, IntoBytes)]
pub struct SetupHeader {
    pub setup_sects: u8,
    pub root_flags: u16,
    pub syssize: u32,
    pub ram_size: u16,
    pub vid_mode: u16,
    pub root_dev: u16,
    pub boot_flag: u16,
    pub jump: u16,
    pub header: u32,
    pub version: u16,
    pub realmode_swtch: u32,
    pub start_sys_seg: u16,
    pub kernel_version: u16,
    pub type_of_loader: u8,
    pub loadflags: u8,
    pub setup_move_size: u16,
    pub code32_start: u32,
    pub ramdisk_image: u32,
    pub ramdisk_size: u32,
    pub bootsect_kludge: u32,
    pub heap_end_ptr: u16,
    pub ext_loader_ver: u8,
    pub ext_loader_type: u8,
    pub cmd_line_ptr: u32,
    pub initrd_addr_max: u32,
    pub kernel_alignment: u32,
    pub relocatable_kernel: u8,
    pub min_alignment: u8,
    pub xloadflags: u16,
    pub cmdline_size: u32,
    pub hardware_subarch: u32,
    pub hardware_subarch_data: u64,
    pub payload_offset: u32,
    pub payload_length: u32,
    pub setup_data: u64,
    pub pref_address: u64,
    pub init_size: u32,
    pub handover_offset: u32,
    pub kernel_info_offset: u32,
}

#[allow(dead_code)]
#[repr(u32)]
enum E820Type {
    Ram = 1,
    Reserved = 2,
    Acpi = 3,
    Nvs = 4,
    Unusable = 5,
    Pmem = 7,
    Pram = 12,
    SoftReserved = 0xefffffff,
    ReservedKern = 128,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros)]
struct BootE820Entry {
    addr: u64,
    size: u64,
    ty: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct InterfacePathIsa {
    base_address: u16,
    reserved1: u16,
    reserved2: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct InterfacePathPci {
    bus: u8,
    slot: u8,
    function: u8,
    channel: u8,
    reserved: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct InterfacePathIbnd {
    reserved: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct InterfacePathXprs {
    reserved: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct InterfacePathHtpt {
    reserved: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct InterfacePathUnknown {
    reserved: u64,
}

#[repr(C, packed)]
#[derive(FromZeros)]
union InterfacePath {
    isa: InterfacePathIsa,
    pci: InterfacePathPci,
    ibnd: InterfacePathIbnd,
    xprs: InterfacePathXprs,
    htpt: InterfacePathHtpt,
    unknown: InterfacePathUnknown,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct DevicePathAta {
    device: u8,
    reserved1: u8,
    reserved2: u16,
    reserved3: u32,
    reserved4: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct DevicePathAtapi {
    device: u8,
    lun: u8,
    reserved1: u8,
    reserved2: u8,
    reserved3: u32,
    reserved4: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct DevicePathScsi {
    id: u16,
    lun: u64,
    reserved1: u16,
    reserved2: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct DevicePathUsb {
    serial_number: u64,
    reserved: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct DevicePathI1394 {
    eui: u64,
    reserved: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct DevicePathFibre {
    wwid: u64,
    lun: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct DevicePathI2o {
    identity_tag: u64,
    reserved: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct DevicePathRaid {
    array_number: u32,
    reserved1: u32,
    reserved2: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct DevicePathSata {
    device: u8,
    reserved1: u8,
    reserved2: u16,
    reserved3: u32,
    reserved4: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromZeros, Immutable)]
struct DevicePathUnknown {
    reserved1: u64,
    reserved2: u64,
}

#[repr(C)]
#[derive(Clone, Copy, FromZeros)]
union DevicePath {
    ata: DevicePathAta,
    atapi: DevicePathAtapi,
    scsi: DevicePathScsi,
    usb: DevicePathUsb,
    i1394: DevicePathI1394,
    fibre: DevicePathFibre,
    i2o: DevicePathI2o,
    raid: DevicePathRaid,
    sata: DevicePathSata,
    unknown: DevicePathUnknown,
}

#[repr(C, packed)]
#[derive(FromZeros)]
struct EddDeviceParams {
    length: u16,
    info_flags: u16,
    num_default_cylinders: u32,
    num_default_heads: u32,
    sectors_per_track: u32,
    number_of_sectors: u64,
    bytes_per_sector: u16,
    dpte_ptr: u32,               /* 0xFFFFFFFF for our purposes */
    key: u16,                    /* = 0xBEDD */
    device_path_info_length: u8, /* = 44 */
    reserved2: u8,
    reserved3: u16,
    host_bus_type: [u8; 4],
    interface_type: [u8; 8],
    interface_path: InterfacePath,
    device_path: DevicePath,
    reserved4: u8,
    checksum: u8,
}

#[repr(C, packed)]
#[derive(FromZeros)]
struct EddInfo {
    device: u8,
    version: u8,
    interface_support: u16,
    legacy_max_cylinder: u16,
    legacy_max_head: u8,
    legacy_sectors_per_track: u8,
    params: EddDeviceParams,
}

#[repr(C, packed)]
#[derive(FromZeros, KnownLayout)]
pub struct BootParams {
    screen_info: ScreenInfo,        // 0x000
    apm_bios_info: ApmBiosInfo,     // 0x040
    _pad2: [u8; 4],                 // 0x054
    tboot_addr: u64,                // 0x058
    ist_info: IstInfo,              // 0x060
    acpi_rsdp_addr: u64,            // 0x070
    _pad3: [u8; 8],                 // 0x078
    hd0_info: [u8; 16],             // 0x080 (obsolete)
    hd1_info: [u8; 16],             // 0x090 (obsolete)
    sys_desc_table: SysDescTable,   // 0x0a0 (obsolete)
    olpc_ofw_header: OlpcOfwHeader, // 0x0b0
    ext_ramdisk_image: u32,         // 0x0c0
    ext_ramdisk_size: u32,          // 0x0c4
    ext_cmd_line_ptr: u32,          // 0x0c8
    _pad4: [u8; 112],               // 0x0cc
    cc_blob_address: u32,           // 0x13c
    edid_info: EdidInfo,            // 0x140
    efi_info: EfiInfo,              // 0x1c0
    alt_mem_k: u32,                 // 0x1e0
    scratch: u32,                   // 0x1e4
    e820_entries: u8,               // 0x1e8
    eddbuf_entries: u8,             // 0x1e9
    edd_mbr_sig_buf_entries: u8,    // 0x1ea
    kbd_status: u8,                 // 0x1eb
    secure_boot: u8,                // 0x1ec
    _pad5: [u8; 2],                 // 0x1ed
    /*
     * The sentinel is set to a nonzero value (0xff) in header.S.
     *
     * A bootloader is supposed to only take setup_header and put
     * it into a clean boot_params buffer. If it turns out that
     * it is clumsy or too generous with the buffer, it most
     * probably will pick up the sentinel variable too. The fact
     * that this variable then is still 0xff will let kernel
     * know that some variables in boot_params are invalid and
     * kernel should zero out certain portions of boot_params.
     */
    sentinel: u8,     // 0x1ef
    _pad6: [u8; 1],   // 0x1f0
    hdr: SetupHeader, // 0x1f1
    _pad7: [u8; 0x290 - 0x1f1 - core::mem::size_of::<SetupHeader>()],
    edd_mbr_sig_buffer: [u32; EDD_MBR_SIG_MAX], // 0x290
    e820_table: [BootE820Entry; E820_MAX_ENTRIES_ZEROPAGE], // 0x2d0
    _pad8: [u8; 48],                            // 0xcd0
    eddbuf: [EddInfo; EDDMAXNR],                // 0xd00
    _pad9: [u8; 276],                           // 0xeec
}

#[derive(Error, Debug)]
pub enum ZeroPageError {
    #[error("acpi_rsdp_addr unset")]
    AcpiRsdpAddrUnset,

    #[error("acpi_rsdp_addr already unset")]
    AcpiRsdpAddrAlreadySet,

    #[error("hdr unset")]
    HdrUnset,

    #[error("hdr already set")]
    HdrAlreadySet,

    #[error("e820_table unset")]
    E820Unset,

    #[error("e820_table already set")]
    E820AlreadySet,
}

#[derive(Default)]
pub struct ZeroPageBuilder {
    acpi_rsdp_addr: OnceCell<u64>,
    hdr: OnceCell<SetupHeader>,
    e820_table: OnceCell<[BootE820Entry; E820_MAX_ENTRIES_ZEROPAGE]>,
    e820_entries: OnceCell<u8>,
}

impl ZeroPageBuilder {
    pub fn setup_acpi_rsdp_addr(self, acpi_rsdp_addr: u64) -> Result<Self, ZeroPageError> {
        self.acpi_rsdp_addr
            .set(acpi_rsdp_addr)
            .map_err(|_| ZeroPageError::AcpiRsdpAddrAlreadySet)?;

        Ok(self)
    }

    pub fn setup_hdr(self, hdr: SetupHeader) -> Result<Self, ZeroPageError> {
        self.hdr
            .set(hdr)
            .map_err(|_| ZeroPageError::HdrAlreadySet)?;

        Ok(self)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn setup_e820(
        self,
        memory: &MemoryAddressSpace,
        acpi_rsdt_addr: u32,
        acpi_max_length: u32,
        mmio_start: u32,
        mmio_length: u32,
        ecam_base: u32,
        ecam_length: u32,
    ) -> Result<Self, ZeroPageError> {
        let mut e820_table = [BootE820Entry::new_zeroed(); E820_MAX_ENTRIES_ZEROPAGE];

        let mut index = 0;

        for region in memory.regions().values() {
            e820_table[index] = BootE820Entry {
                addr: region.gpa,
                size: region.len() as u64,
                ty: E820Type::Ram as u32,
            };
            index += 1;
        }

        e820_table[index] = BootE820Entry {
            addr: acpi_rsdt_addr as u64,
            size: acpi_max_length as u64,
            ty: E820Type::Acpi as u32,
        };
        index += 1;

        e820_table[index] = BootE820Entry {
            addr: mmio_start as u64,
            size: mmio_length as u64,
            ty: E820Type::Reserved as u32,
        };
        index += 1;

        e820_table[index] = BootE820Entry {
            addr: ecam_base as u64,
            size: ecam_length as u64,
            ty: E820Type::Reserved as u32,
        };
        index += 1;

        self.e820_table
            .set(e820_table)
            .map_err(|_| ZeroPageError::E820AlreadySet)?;
        self.e820_entries
            .set(index as u8)
            .map_err(|_| ZeroPageError::E820AlreadySet)?;

        Ok(self)
    }

    pub fn build(mut self) -> Result<BootParams, ZeroPageError> {
        let boot_params = BootParams {
            acpi_rsdp_addr: self
                .acpi_rsdp_addr
                .take()
                .ok_or(ZeroPageError::AcpiRsdpAddrUnset)?,
            hdr: self.hdr.take().ok_or(ZeroPageError::HdrUnset)?,
            e820_table: self.e820_table.take().ok_or(ZeroPageError::E820Unset)?,
            e820_entries: self.e820_entries.take().ok_or(ZeroPageError::E820Unset)?,
            ..BootParams::new_zeroed()
        };

        Ok(boot_params)
    }
}
