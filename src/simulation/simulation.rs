use std::time::Duration;

use super::{particles::Particles, sim_events::SimEventsHandler, systems::System, types::Time};
use crate::utils::perf_logger::PerformanceLogger;

pub struct Simulation {
    pub particles: Particles,
    pub systems: Vec<Box<dyn System>>,
    pub event_handler: Option<Box<dyn SimEventsHandler>>,
    logger: PerformanceLogger,
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
            logger: PerformanceLogger::new(Duration::from_secs(1)),
        }
    }

    pub fn step(&mut self, dt: Time) {
        self.logger.start();

        // Update events
        if let Some(event_handler) = &mut self.event_handler {
            event_handler.update(&mut self.particles, &mut self.systems, dt);

            self.logger.time("Event Handler");
        }

        // Update systems
        for (i, system) in &mut self.systems.iter_mut().enumerate() {
            system.update(&mut self.particles, dt);

            self.logger.time(&format!("[{}] {}", i, system.get_name()));
        }

        self.logger.stop();
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
