use strum_macros::FromRepr;
use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;

#[derive(FromRepr)]
pub enum VirtioBalloonTranditionalVirtqueue {
    Inflateq = 0,
    Defalteq = 1,
    Statsq = 2,
    FreePageVq = 3,
    ReportingVq = 4,
}

#[allow(non_camel_case_types)]
pub enum VirtioBalloonTranditionalFeatureBitmap {
    MUST_TELL_HOST = 0, /* Tell before reclaiming pages */
    STATS_VQ = 1,       /* Memory Stats virtqueue */
    DEFLATE_ON_OOM = 2, /* Deflate balloon on OOM */
    FREE_PAGE_HINT = 3, /* VQ to report free pages */
    PAGE_POISON = 4,    /* Guest is using page poisoning */
    REPORTING = 5,      /* Page reporting virtqueue */
}

#[derive(Default, FromBytes, IntoBytes, Immutable)]
#[repr(C, packed)]
pub struct VirtioBalloonTranditionalConfig {
    pub num_pages: u32,
    pub actual: u32,
    pub free_page_hint_cmd_id: u32,
    pub poison_val: u32,
}
