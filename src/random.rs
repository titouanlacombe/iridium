use rand::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

pub struct RngGenerator {
    rng: XorShiftRng,
}

// Use XorShiftRng to generate XorShiftRng
impl RngGenerator {
    pub fn new(seed: u128) -> Self {
        Self {
            rng: XorShiftRng::from_seed(seed.to_le_bytes()),
        }
    }

    pub fn next(&mut self) -> XorShiftRng {
        let mut seed = [0; 16];
        self.rng.fill_bytes(&mut seed);
        XorShiftRng::from_seed(seed)
    }
}
