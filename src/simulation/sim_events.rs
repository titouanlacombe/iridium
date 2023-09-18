use super::{particles::Particles, systems::System, types::Time};
use crate::utils::sorted_vec::SortedVec;

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

impl PartialEq for SimEvent {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

// Sort from latest to earliest
impl PartialOrd for SimEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.time.partial_cmp(&self.time)
    }
}

pub trait SimEventsHandler {
    fn update(&mut self, particles: &mut Particles, systems: &mut Vec<Box<dyn System>>, dt: Time);
}

pub struct DefaultSimEventsHandler {
    pub events: SortedVec<SimEvent>,
    pub current_time: Time,
}

impl DefaultSimEventsHandler {
    pub fn new(events: SortedVec<SimEvent>, current_time: Time) -> Self {
        Self {
            events,
            current_time,
        }
    }
}

impl SimEventsHandler for DefaultSimEventsHandler {
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
