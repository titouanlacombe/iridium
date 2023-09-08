use std::{
    collections::{hash_map::Entry, HashMap},
    time::{Duration, Instant},
};

use log::info;

use crate::timer::Timer;

// Warning this file is ugly, i just wanted to get it done
pub struct PeriodicToggle {
    interval: Duration,

    // Variables
    last: Instant,
}

impl PeriodicToggle {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last: Instant::now(),
        }
    }

    pub fn get(&mut self) -> bool {
        let now = Instant::now();
        if now - self.last < self.interval {
            return false;
        }
        self.last = now;
        true
    }
}

pub struct RunningAverage<T> {
    value: T,
    len: usize,
}

impl<T: Default + Copy + std::ops::Div<f64, Output = T> + std::ops::AddAssign> RunningAverage<T> {
    pub fn new() -> Self {
        Self {
            value: T::default(),
            len: 0,
        }
    }

    pub fn add(&mut self, value: T) {
        self.value += value;
        self.len += 1;
    }

    pub fn get(&self) -> T {
        self.value / self.len as f64
    }
}

pub struct PerformanceLogger {
    log_toggle: PeriodicToggle,
    log_data: HashMap<String, RunningAverage<f64>>,
    insertion_order: Vec<String>,
    timer: Timer,
}

impl PerformanceLogger {
    pub fn new(interval: Duration) -> Self {
        Self {
            log_toggle: PeriodicToggle::new(interval),
            log_data: HashMap::new(),
            insertion_order: Vec::new(),
            timer: Timer::new_now(),
        }
    }

    // Reset timer at the start of the measurements
    pub fn start(&mut self) {
        self.timer.lap();
    }

    // Call to time "name" since last call
    pub fn time(&mut self, name: &str) {
        match self.log_data.entry(name.to_string()) {
            Entry::Occupied(entry) => {
                entry.into_mut().add(self.timer.lap().as_secs_f64());
            }
            Entry::Vacant(entry) => {
                self.insertion_order.push(name.to_string());
                entry.insert(RunningAverage::new());
            }
        };
    }

    // Call at the end of the measurements
    pub fn stop(&mut self) {
        // Log data if toggle is on
        if self.log_toggle.get() {
            for key in self.insertion_order.iter() {
                let value = self.log_data.get(key).unwrap();
                info!("{}: {} us", key, (value.get() * 10E6).round());
            }
            self.log_data.clear();
            self.insertion_order.clear();
        }
    }
}
