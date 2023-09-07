use sfml::{
    system::Vector2i,
    window::{
        mouse::{Button, Wheel},
        Event,
    },
};
use std::{rc::Rc, sync::RwLock};

use crate::{
    coordinates::CoordinateSystem, render_thread::RenderThread, renderer::Renderer,
    simulation::Simulation, types::Position,
};

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

pub type UserEventCallback =
    Box<dyn FnMut(&mut Box<dyn Renderer>, &mut Simulation, &mut bool, &UserEvent)>;

pub trait UserEventHandler {
    fn handle_events(
        &mut self,
        renderer: &mut Box<dyn Renderer>,
        sim: &mut Simulation,
        running: &mut bool,
    );
}

pub struct BasicUserEventHandler {
    render_thread: Rc<RenderThread>,
    callback: UserEventCallback,
    coord_system: Rc<RwLock<dyn CoordinateSystem>>,
}

impl BasicUserEventHandler {
    pub fn new(
        render_thread: Rc<RenderThread>,
        callback: UserEventCallback,
        coord_system: Rc<RwLock<dyn CoordinateSystem>>,
    ) -> Self {
        Self {
            render_thread,
            callback,
            coord_system,
        }
    }

    fn convert_event(&self, event: &Event) -> UserEvent {
        let mut position = None;

        // Match mouse events
        match event {
            Event::MouseButtonPressed { x, y, .. }
            | Event::MouseButtonReleased { x, y, .. }
            | Event::MouseMoved { x, y }
            | Event::MouseWheelScrolled { x, y, .. }
            | Event::TouchBegan { x, y, .. }
            | Event::TouchMoved { x, y, .. }
            | Event::TouchEnded { x, y, .. } => {
                position = Some(Vector2i::new(*x, *y));
            }
            _ => (),
        }

        if position.is_none() {
            return UserEvent::Event(*event);
        }

        let position = self
            .coord_system
            .read()
            .unwrap()
            .screen2sim(position.unwrap());

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
}

impl UserEventHandler for BasicUserEventHandler {
    fn handle_events(
        &mut self,
        renderer: &mut Box<dyn Renderer>,
        sim: &mut Simulation,
        running: &mut bool,
    ) {
        let events = self.render_thread.get_events();

        events.iter().for_each(|event| {
            let event = self.convert_event(event);
            (self.callback)(renderer, sim, running, &event);
        });
    }
}
