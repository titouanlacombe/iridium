use sfml::window::{
    mouse::{Button, Wheel},
    Event,
};
use std::rc::Rc;

use crate::{input::KeysState, iridium::SimData, render_thread::RenderThread, types::Position};

pub trait InputHandler {
    fn handle(&mut self, data: &mut SimData, real_dt: f64);
}

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

pub type UserEventCallback = Box<dyn FnMut(&mut SimData, &UserEvent)>;
pub type FrameCallback = Box<dyn FnMut(&mut SimData, &KeysState, f64)>;

pub struct BasicInputHandler {
    render_thread: Rc<RenderThread>,
    keys_state: KeysState,
    event_callback: UserEventCallback,
    frame_callback: FrameCallback,
}

impl BasicInputHandler {
    pub fn new(
        render_thread: Rc<RenderThread>,
        event_callback: UserEventCallback,
        frame_callback: FrameCallback,
    ) -> Self {
        Self {
            render_thread,
            keys_state: KeysState::new(),
            event_callback,
            frame_callback,
        }
    }
}

impl InputHandler for BasicInputHandler {
    fn handle(&mut self, data: &mut SimData, real_dt: f64) {
        self.render_thread.get_events().iter().for_each(|event| {
            self.keys_state.update(event);
            (self.event_callback)(data, event);
        });
        (self.frame_callback)(data, &self.keys_state, real_dt);
    }
}
