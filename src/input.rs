use nalgebra::Vector2;
use sfml::window::{Event, Key};
use std::collections::HashMap;

use crate::{camera::Camera, user_events::UserEvent};

pub struct KeysState {
    map: HashMap<Key, bool>,
}

impl KeysState {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn update(&mut self, event: &UserEvent) {
        match event {
            UserEvent::Event(event) => match event {
                Event::KeyPressed { code, .. } => {
                    self.map.insert(*code, true);
                }
                Event::KeyReleased { code, .. } => {
                    self.map.insert(*code, false);
                }
                _ => (),
            },
            _ => (),
        }
    }

    pub fn get(&self, key: Key) -> bool {
        *self.map.get(&key).unwrap_or(&false)
    }
}

pub fn sfml2user_event(event: &Event, camera: &dyn Camera) -> UserEvent {
    let mut screen_pos = None;

    // Match positioned events
    match event {
        Event::MouseButtonPressed { x, y, .. }
        | Event::MouseButtonReleased { x, y, .. }
        | Event::MouseMoved { x, y }
        | Event::MouseWheelScrolled { x, y, .. }
        | Event::TouchBegan { x, y, .. }
        | Event::TouchMoved { x, y, .. }
        | Event::TouchEnded { x, y, .. } => {
            screen_pos = Some(Vector2::new(*x as f64, *y as f64));
        }
        _ => (),
    }

    // Return event if no position (no need to convert)
    if screen_pos.is_none() {
        return UserEvent::Event(*event);
    }

    let position = camera.screen2sim(screen_pos.unwrap());

    match event {
        Event::MouseButtonPressed { button, .. } => UserEvent::MouseButtonPressed {
            button: *button,
            position,
        },
        Event::MouseButtonReleased { button, .. } => UserEvent::MouseButtonReleased {
            button: *button,
            position,
        },
        Event::MouseMoved { .. } => UserEvent::MouseMoved { position },
        Event::MouseWheelScrolled { wheel, delta, .. } => UserEvent::MouseWheelScrolled {
            wheel: *wheel,
            delta: *delta,
            position,
        },
        Event::TouchBegan { finger, .. } => UserEvent::TouchBegan {
            finger: *finger,
            position,
        },
        Event::TouchMoved { finger, .. } => UserEvent::TouchMoved {
            finger: *finger,
            position,
        },
        Event::TouchEnded { finger, .. } => UserEvent::TouchEnded {
            finger: *finger,
            position,
        },
        _ => UserEvent::Event(*event),
    }
}
