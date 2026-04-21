pub(crate) struct Dispstat(pub(crate) u16);

impl Dispstat {
    pub(crate) fn vcount_setting(&self) -> u16 {
        self.0 >> 7
    }
}
