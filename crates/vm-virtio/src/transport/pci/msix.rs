use vm_pci::device::capability::msix::MsixEntry;

/*
 * 4.1.5.1.2.1 Device Requirements: MSI-X Vector Configuration
 *   A device that has an MSI-X capability SHOULD support at least 2 and at most 0x800 MSI-X vectors.
 */
const MSI_X_VECTORS_MIN: u16 = 2;
const MSI_X_VECTORS_MAX: u16 = 0x800;

pub struct VirtioPciMsixInfo {
    // The number of MSI-X vectors
    vectors: u16,
    pub table: Vec<MsixEntry>,
}

impl VirtioPciMsixInfo {
    pub fn new(num_queues: u16) -> Self {
        /*
         * Refer to linux: `vp_find_vqs_msix`:
         *   Best option: one for change interrupt, one per vq.
         */
        let vectors = (num_queues + 1).clamp(MSI_X_VECTORS_MIN, MSI_X_VECTORS_MAX);
        let table = (0..vectors).map(|_| MsixEntry::default()).collect();
        VirtioPciMsixInfo { vectors, table }
    }

    pub fn vectors(&self) -> u16 {
        self.vectors
    }

    pub fn pba_offset(&self) -> u32 {
        u32::try_from(self.vectors as usize * size_of::<MsixEntry>())
            .unwrap()
            .next_multiple_of(8)
    }

    pub fn bar_size(&self) -> u32 {
        (self.pba_offset() + ((self.vectors() as u32).div_ceil(8))).next_multiple_of(4096)
    }
}
