use log::debug;
use nalgebra::Vector2;

use crate::{
    forces::{UniformDrag, UniformGravity},
    particle::Particle,
    systems::System,
    timer::Timer,
};

pub struct Simulation {
    pub particles: Vec<Particle>,
    pub systems: Vec<Box<dyn System>>,

    uniform_gravity: Option<UniformGravity>,
    uniform_drag: Option<UniformDrag>,
}

impl Simulation {
    pub fn new(
        particles: Vec<Particle>,
        systems: Vec<Box<dyn System>>,
        uniform_gravity: Option<UniformGravity>,
        uniform_drag: Option<UniformDrag>,
    ) -> Self {
        Self {
            particles,
            systems,
            uniform_gravity,
            uniform_drag,
        }
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

        // Update particles
        for particle in &mut self.particles {
            let mut forces: Vector2<f32> = Vector2::new(0., 0.);

            // Apply forces
            if let Some(gravity) = &self.uniform_gravity {
                gravity.apply(particle, &mut forces);
            }

            if let Some(drag) = &self.uniform_drag {
                drag.apply(particle, &mut forces);
            }

            // Update particle
            particle.velocity += forces * dt / particle.mass;
            particle.position += particle.velocity * dt;
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
