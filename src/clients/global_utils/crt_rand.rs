pub(crate) struct CrtRand {
    seed: u32
}

impl CrtRand {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }
    pub fn next(&mut self) -> u32 {
        self.seed = 0x343FD * self.seed + 0x269EC3;
        ((self.seed >> 16) & 0xFFFF) & 0x7FFF
    }
}