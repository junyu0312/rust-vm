use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

#[derive(FromBytes, IntoBytes, KnownLayout)]
#[repr(C, packed)]
pub struct VirtIoPciCap {
    pub cap_vndr: u8, /* Generic PCI field: PCI_CAP_ID_VNDR */
    pub cap_next: u8, /* Generic PCI field: next ptr. */
    pub cap_len: u8,  /* Generic PCI field: capability length */
    pub cfg_type: u8, /* Identifies the structure. */
    pub bar: u8,      /* Where to find it. */
    pub id: u8,       /* Multiple capabilities of the same type */
    padding: [u8; 2], /* Pad to full dword. */
    pub offset: u32,  /* Offset within bar. */
    pub length: u32,  /* Length of the structure, in bytes. */
}

#[repr(u8)]
pub enum VirtIoPciCapCfgType {
    /* Common configuration */
    VirtioPciCapCommonCfg = 1,
    /* Notifications */
    VirtioPciCapNotifyCfg = 2,
    /* ISR Status */
    VirtioPciCapIsrCfg = 3,
    /* Device specific configuration */
    VirtioPciCapDeviceCfg = 4,
    /* PCI configuration access */
    VirtioPciCapPciCfg = 5,
    /* Shared memory region */
    VirtioPciCapSharedMemoryCfg = 8,
    /* Vendor-specific data */
    VirtioPciCapVendorCfg = 9,
}

#[derive(Default, FromBytes, IntoBytes, Immutable)]
#[repr(C, packed)]
pub struct VirtIoPciCommonCfg {
    /* About the whole device. */
    pub device_feature_select: u32,
    pub device_feature: u32,
    pub driver_feature_select: u32,
    pub driver_feature: u32,
    pub config_msix_vector: u16,
    pub num_queues: u16,
    pub device_status: u8,
    pub config_generation: u8,

    /* About a specific virtqueue. */
    pub queue_select: u16,
    pub queue_size: u16,
    pub queue_msix_vector: u16,
    pub queue_enable: u16,
    pub queue_notify_off: u16,
    pub queue_desc: u64,
    pub queue_driver: u64,
    pub queue_device: u64,
    pub queue_notif_config_data: u16,
    pub queue_reset: u16,
    pub admin_queue_index: u16,
    pub admin_queue_num: u16,
}

#[derive(FromBytes, IntoBytes, KnownLayout)]
#[repr(C, packed)]
pub struct VirtIoPciNotifyCap {
    pub cap: VirtIoPciCap,
    pub notify_off_multiplier: u32,
}
