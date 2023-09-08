use nalgebra::Vector2;
use sfml::window::{
    mouse::{Button, Wheel},
    Event,
};
use std::rc::Rc;

use crate::{camera::Camera, iridium::SimData, render_thread::RenderThread, types::Position};

pub enum UserEvent {
    Event(Event),
    // Custom user events (abstracts window handling, directly in simulation coordinates)
    MouseButtonPressed {
        button: Button,
        position: Position,
    },
    MouseButtonReleased {
        button: Button,
        position: Position,
    },
    MouseMoved {
        position: Position,
    },
    MouseWheelScrolled {
        wheel: Wheel,
        delta: f32,
        position: Position,
    },
    TouchBegan {
        finger: u32,
        position: Position,
    },
    TouchMoved {
        finger: u32,
        position: Position,
    },
    TouchEnded {
        finger: u32,
        position: Position,
    },
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

pub type UserEventCallback = Box<dyn FnMut(&mut SimData, &UserEvent)>;

pub trait UserEventHandler {
    fn handle_events(&mut self, data: &mut SimData);
}

pub struct BasicUserEventHandler {
    render_thread: Rc<RenderThread>,
    callback: UserEventCallback,
}

impl BasicUserEventHandler {
    pub fn new(render_thread: Rc<RenderThread>, callback: UserEventCallback) -> Self {
        Self {
            render_thread,
            callback,
        }
    }
}

impl UserEventHandler for BasicUserEventHandler {
    fn handle_events(&mut self, data: &mut SimData) {
        self.render_thread.get_events().iter().for_each(|event| {
            (self.callback)(data, event);
        });
    }
}
