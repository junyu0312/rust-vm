pub struct DriverFeatures<const N: usize = 2>([u32; N]);

impl<const N: usize> Default for DriverFeatures<N> {
    fn default() -> Self {
        DriverFeatures([0; N])
    }
}

impl<const N: usize> DriverFeatures<N> {
    pub fn write(&mut self, sel: usize, feat: u32) {
        self.0[sel] = feat;
    }
}
