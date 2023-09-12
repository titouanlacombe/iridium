use std::time::{Duration, Instant};

pub struct Timer {
    pub start: Instant,
}

impl Timer {
    pub fn new(start: Instant) -> Self {
        Self { start }
    }

    pub fn new_now() -> Self {
        Self::new(Instant::now())
    }

    pub fn reset_to(&mut self, time: Instant) {
        self.start = time;
    }

    pub fn reset(&mut self) {
        self.reset_to(Instant::now());
    }

    pub fn elapsed_from(&self, time: Instant) -> Duration {
        time - self.start
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed_from(Instant::now())
    }

    // Elapse & reset but with only one call to Instant::now() instead of two
    pub fn lap(&mut self) -> Duration {
        let now = Instant::now();
        let elapsed = self.elapsed_from(now);
        self.reset_to(now);
        elapsed
    }
}
