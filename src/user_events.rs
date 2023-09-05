use sfml::window::Event;
use std::sync::{Arc, Mutex};

use crate::{
    render_thread::GetEvents,
    renderer::{RenderThreadHandle, Renderer},
    simulation::Simulation,
};

pub type EventCallback = Box<dyn FnMut(&mut Box<dyn Renderer>, &mut Simulation, &mut bool, &Event)>;

pub trait UserEventHandler {
    fn handle_events(
        &mut self,
        renderer: &mut Box<dyn Renderer>,
        sim: &mut Simulation,
        running: &mut bool,
    );
}

pub struct BasicUserEventHandler {
    render_thread: Arc<Mutex<RenderThreadHandle>>,
    event_callback: EventCallback,
}

impl BasicUserEventHandler {
    pub fn new(
        render_thread: Arc<Mutex<RenderThreadHandle>>,
        event_callback: EventCallback,
    ) -> Self {
        Self {
            render_thread,
            event_callback,
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
        GetEvents
            .send(&self.render_thread.lock().unwrap().channel)
            .recv()
            .unwrap()
            .iter()
            .for_each(|event| {
                (self.event_callback)(renderer, sim, running, event);
            });
    }
}
