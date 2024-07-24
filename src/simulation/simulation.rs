use super::{particles::Particles, sim_events::SimEventsHandler, systems::System, types::Time};

pub struct Simulation {
    pub particles: Particles,
    pub systems: Vec<Box<dyn System>>,
    pub event_handler: Option<Box<dyn SimEventsHandler>>,
}

impl Simulation {
    pub fn new(
        particles: Particles,
        systems: Vec<Box<dyn System>>,
        event_handler: Option<Box<dyn SimEventsHandler>>,
    ) -> Self {
        Self {
            particles,
            systems,
            event_handler,
        }
    }

    pub fn step(&mut self, dt: Time) {
        // Update events
        if let Some(event_handler) = &mut self.event_handler {
            let _span = tracy_client::span!("Event Handler");
            event_handler.update(&mut self.particles, &mut self.systems, dt);
        }

        // Update systems
        for (i, system) in &mut self.systems.iter_mut().enumerate() {
            let span = tracy_client::span!("System");
            span.emit_text(&format!("[{}] {}", i, system.type_name()));
            system.update(&mut self.particles, dt);
        }
    }
}

pub trait SimulationRunner {
    fn step(&mut self, sim: &mut Simulation);
}

pub struct ConstantSimulationRunner {
    dt: Time,
}

impl ConstantSimulationRunner {
    pub fn new(dt: Time) -> Self {
        Self { dt }
    }
}

impl SimulationRunner for ConstantSimulationRunner {
    fn step(&mut self, sim: &mut Simulation) {
        sim.step(self.dt);
    }
}
