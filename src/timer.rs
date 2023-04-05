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

    pub fn reset_time(&mut self, time: Instant) {
        self.start = time;
    }

    pub fn elapsed_time(&self, time: Instant) -> Duration {
        time - self.start
    }

    // Elapse & reset but with only one call to Instant::now() instead of two
    pub fn lap(&mut self) -> Duration {
        let now = Instant::now();
        let elapsed = self.elapsed_time(now);
        self.reset_time(now);
        elapsed
    }
}
