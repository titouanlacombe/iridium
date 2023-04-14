use log::debug;

use crate::{particle::Particles, systems::System, timer::Timer};

pub struct Simulation {
    pub particles: Particles,
    pub systems: Vec<Box<dyn System>>,
}

impl Simulation {
    pub fn new(particles: Particles, systems: Vec<Box<dyn System>>) -> Self {
        Self { particles, systems }
    }

    pub fn step(&mut self, dt: f32) {
        let mut timer = Timer::new_now();

        // Update systems
        for (i, system) in &mut self.systems.iter_mut().enumerate() {
            system.update(&mut self.particles, dt);
            debug!(
                "System {} update took {:.2} ms",
                i,
                timer.lap().as_secs_f64() * 1000.,
            );
        }
    }
}

pub trait SimulationRunner {
    fn step(&mut self, sim: &mut Simulation);
}

pub struct ContinuousSimulationRunner {
    dt: f32,
}

impl ContinuousSimulationRunner {
    pub fn new(dt: f32) -> Self {
        Self { dt }
    }
}

impl SimulationRunner for ContinuousSimulationRunner {
    fn step(&mut self, sim: &mut Simulation) {
        sim.step(self.dt);
    }
}
