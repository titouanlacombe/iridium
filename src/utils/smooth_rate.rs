// Handle the case where the rate is not an integer
// Making the rate smooth & accurate across steps
// Use a remainder to be deterministic
pub struct SmoothRate {
    rate: f64,
    remainder: f64,
}

impl SmoothRate {
    pub fn new(rate: f64) -> Self {
        Self {
            rate,
            remainder: 0.0,
        }
    }

    pub fn get(&mut self, dt: f64) -> usize {
        let n = self.rate * dt + self.remainder;
        let quotient = n as usize;

        // Remainder
        self.remainder = n - quotient as f64;

        quotient
    }
}
