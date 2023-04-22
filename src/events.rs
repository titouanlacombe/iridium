use crate::{particle::Particles, systems::System, types::Time};

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

type EventCallback = Box<dyn Fn(&mut Particles, &mut Vec<Box<dyn System>>)>;
pub struct Event {
    pub time: Time,
    pub callback: EventCallback,
}

impl Event {
    pub fn new(time: Time, callback: EventCallback) -> Self {
        Self { time, callback }
    }
}

// Sort from latest to earliest
impl PartialEq for Event {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.time.partial_cmp(&self.time)
    }
}

pub trait EventsHandler {
    fn update(&mut self, particles: &mut Particles, systems: &mut Vec<Box<dyn System>>, dt: Time);
}

pub struct DefaultEventsHandler {
    pub events: SortedVec<Event>,
    pub current_time: Time,
}

impl DefaultEventsHandler {
    pub fn new(events: SortedVec<Event>, current_time: Time) -> Self {
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
