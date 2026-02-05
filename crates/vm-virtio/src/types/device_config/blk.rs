pub mod features {
    pub const VIRTIO_BLK_F_SIZE_MAX: u32 = 1;
    pub const VIRTIO_BLK_F_SEG_MAX: u32 = 2;
    pub const VIRTIO_BLK_F_GEOMETRY: u32 = 4;
    pub const VIRTIO_BLK_F_RO: u32 = 5;
    pub const VIRTIO_BLK_F_BLK_SIZE: u32 = 6;
    pub const VIRTIO_BLK_F_FLUSH: u32 = 9;
    pub const VIRTIO_BLK_F_TOPOLOGY: u32 = 10;
    pub const VIRTIO_BLK_F_CONFIG_WCE: u32 = 11;
    pub const VIRTIO_BLK_F_MQ: u32 = 12;
    pub const VIRTIO_BLK_F_DISCARD: u32 = 13;
    pub const VIRTIO_BLK_F_WRITE_ZEROES: u32 = 14;
    pub const VIRTIO_BLK_F_LIFETIME: u32 = 15;
    pub const VIRTIO_BLK_F_SECURE_ERASE: u32 = 16;
    pub const VIRTIO_BLK_F_ZONED: u32 = 17;
}

pub mod config {
    use zerocopy::FromBytes;
    use zerocopy::Immutable;
    use zerocopy::IntoBytes;

    #[derive(Default, FromBytes, IntoBytes, Immutable)]
    #[repr(C)]
    pub struct VirtioBlkConfig {
        pub capacity: u64,

        pub size_max: u32,
        pub seg_max: u32,
        pub geometry: VirtioBlkGeometry,
        pub blk_size: u32,
        pub topology: VirtioBlkTopology,
        pub writeback: u8,
        pub unused0: u8,
        pub num_queues: u16,

        pub max_discard_sectors: u32,
        pub max_discard_seg: u32,
        pub discard_sector_alignment: u32,

        pub max_write_zeroes_sectors: u32,
        pub max_write_zeroes_seg: u32,
        pub write_zeroes_may_unmap: u8,
        pub unused1: [u8; 3],

        pub max_secure_erase_sectors: u32,
        pub max_secure_erase_seg: u32,
        pub secure_erase_sector_alignment: u32,

        pub zoned: VirtioBlkZonedCharacteristics,
    }

    #[derive(Default, FromBytes, IntoBytes, Immutable)]
    #[repr(C)]
    pub struct VirtioBlkGeometry {
        pub cylinders: u16,
        pub heads: u8,
        pub sectors: u8,
    }

    #[derive(Default, FromBytes, IntoBytes, Immutable)]
    #[repr(C)]
    pub struct VirtioBlkTopology {
        pub physical_block_exp: u8,
        pub alignment_offset: u8,
        pub min_io_size: u16,
        pub opt_io_size: u32,
    }

    #[derive(Default, FromBytes, IntoBytes, Immutable)]
    #[repr(C)]
    pub struct VirtioBlkZonedCharacteristics {
        pub zone_sectors: u32,
        pub max_open_zones: u32,
        pub max_active_zones: u32,
        pub max_append_sectors: u32,
        pub write_granularity: u32,
        pub model: u8,
        pub unused2: [u8; 3],
    }
}
