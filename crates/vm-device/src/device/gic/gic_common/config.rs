pub struct GicConfig {
    pub cpu_number: usize,

    // distributor
    pub distributor_base: u64,
    pub are: bool, // affinity routing enable
    pub mbis: bool,
    pub security_extn: bool,
    pub nmi: bool,
    pub extended_spi: bool,

    // redistributor
    pub redistributor_base: u64,
    pub redist_stride: Option<usize>,
    pub vlpis: bool,
}
