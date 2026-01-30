/* v1.0 compliant. */
pub const VIRTIO_F_VERSION_1: u32 = 32;

pub struct DeviceFeatures<const N: usize = 2>([u32; N]);

impl<const N: usize> DeviceFeatures<N> {
    pub fn from_u64(feat: u64) -> Self {
        let mut features = [0; N];
        features[0] = feat as u32;
        features[1] = (feat >> 32) as u32;
        DeviceFeatures(features)
    }

    pub fn read(&self, sel: usize) -> u32 {
        self.0[sel]
    }
}
