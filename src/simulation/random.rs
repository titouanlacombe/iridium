use rand::{RngCore, SeedableRng};
use rand_pcg::Pcg64Mcg;

pub struct RngGenerator {
    rng: Pcg64Mcg,
}

// Use XorShiftRng to generate XorShiftRng
impl RngGenerator {
    pub fn new(seed: u128) -> Self {
        Self {
            rng: Pcg64Mcg::from_seed(seed.to_le_bytes()),
        }
    }

    pub fn next(&mut self) -> Pcg64Mcg {
        let mut seed = [0; 16];
        self.rng.fill_bytes(&mut seed);
        Pcg64Mcg::from_seed(seed)
    }
}
