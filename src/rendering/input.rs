use nalgebra::Vector2;
use sfml::{
    graphics::{RenderTarget, RenderWindow},
    system::Vector2i,
    window::{Event, Key},
};
use std::collections::HashMap;

pub struct KeysState {
    map: HashMap<Key, bool>,
}

impl KeysState {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn update(&mut self, event: &WindowEvent) {
        match event.original {
            Event::KeyPressed { code, .. } => {
                self.map.insert(code, true);
            }
            Event::KeyReleased { code, .. } => {
                self.map.insert(code, false);
            }
            _ => (),
        }
    }

    pub fn get(&self, key: Key) -> bool {
        *self.map.get(&key).unwrap_or(&false)
    }
}

pub struct WindowEvent {
    pub original: Event,
    pub position: Option<Vector2<f64>>,
}

impl WindowEvent {
    pub fn from_sfml(event: &Event, window: &RenderWindow) -> Self {
        // Check if the event has a position
        let screen_pos = match event {
            Event::MouseButtonPressed { x, y, .. }
            | Event::MouseButtonReleased { x, y, .. }
            | Event::MouseMoved { x, y }
            | Event::MouseWheelScrolled { x, y, .. }
            | Event::TouchBegan { x, y, .. }
            | Event::TouchMoved { x, y, .. }
            | Event::TouchEnded { x, y, .. } => {
                let position = window.map_pixel_to_coords_current_view(Vector2i::new(*x, *y));
                Some(Vector2::new(position.x as f64, position.y as f64))
            }
            _ => None,
        };

        WindowEvent {
            original: event.clone(),
            position: screen_pos,
        }
    }
}
