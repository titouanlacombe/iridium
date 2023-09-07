use crate::{particles::Particles, systems::System, types::Time};

pub struct SortedVec<T> {
    pub vec: Vec<T>,
}

impl<T: PartialOrd> SortedVec<T> {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }

    pub fn add(&mut self, value: T) {
        let mut left = 0;
        let mut right = self.vec.len();

        while left < right {
            let mid = (left + right) / 2;

            if self.vec[mid] < value {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        self.vec.insert(left, value);
    }

    pub fn first(&self) -> Option<&T> {
        self.vec.first()
    }

    pub fn pop(&mut self) -> Option<T> {
        self.vec.pop()
    }
}

type SimEventCallback = Box<dyn Fn(&mut Particles, &mut Vec<Box<dyn System>>)>;
pub struct SimEvent {
    pub time: Time,
    pub callback: SimEventCallback,
}

impl SimEvent {
    pub fn new(time: Time, callback: SimEventCallback) -> Self {
        Self { time, callback }
    }
}

// Sort from latest to earliest
impl PartialEq for SimEvent {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl PartialOrd for SimEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.time.partial_cmp(&self.time)
    }
}

pub trait EventsHandler {
    fn update(&mut self, particles: &mut Particles, systems: &mut Vec<Box<dyn System>>, dt: Time);
}

pub struct DefaultEventsHandler {
    pub events: SortedVec<SimEvent>,
    pub current_time: Time,
}

impl DefaultEventsHandler {
    pub fn new(events: SortedVec<SimEvent>, current_time: Time) -> Self {
        Self {
            events,
            current_time,
        }
    }
}

impl EventsHandler for DefaultEventsHandler {
    fn update(&mut self, particles: &mut Particles, systems: &mut Vec<Box<dyn System>>, dt: Time) {
        self.current_time += dt;

        while let Some(event) = self.events.first() {
            if event.time > self.current_time {
                break;
            }

            (event.callback)(particles, systems);
            self.events.pop();
        }
    }
}
