// Handle the case where the rate is not an integer
// Making the rate smooth & accurate across steps
// Use a remainder to be deterministic
use crate::simulation::types::Scalar;

pub struct SmoothRate {
    rate: Scalar,
    remainder: Scalar,
}

impl SmoothRate {
    pub fn new(rate: Scalar) -> Self {
        Self {
            rate,
            remainder: 0.0,
        }
    }

    pub fn get(&mut self, dt: Scalar) -> usize {
        let n = self.rate * dt + self.remainder;
        let quotient = n as usize;

        // Remainder
        self.remainder = n - quotient as Scalar;

        quotient
    }
}
