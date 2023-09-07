use log::debug;

use crate::{
    particles::Particles, sim_events::SimEventsHandler, systems::System, timer::Timer, types::Time,
};

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
        let mut timer = Timer::new_now();

        // Update events
        if let Some(event_handler) = &mut self.event_handler {
            event_handler.update(&mut self.particles, &mut self.systems, dt);
            debug!(
                "Sim event handler update took {} us",
                timer.lap().as_micros(),
            );
        }

        // Update systems
        for (i, system) in &mut self.systems.iter_mut().enumerate() {
            system.update(&mut self.particles, dt);
            debug!("System {} update took {} us", i, timer.lap().as_micros());
        }
    }
}

pub trait SimulationRunner {
    fn step(&mut self, sim: &mut Simulation);
}

pub struct ContinuousSimulationRunner {
    dt: Time,
}

impl ContinuousSimulationRunner {
    pub fn new(dt: Time) -> Self {
        Self { dt }
    }
}

impl SimulationRunner for ContinuousSimulationRunner {
    fn step(&mut self, sim: &mut Simulation) {
        sim.step(self.dt);
    }
}
