use nalgebra::Vector2;

use crate::{
    forces::{UniformDrag, UniformGravity},
    particle::Particle,
    systems::System,
};

pub struct Simulation {
    pub particles: Vec<Particle>,
    pub systems: Vec<Box<dyn System>>,

    // TODO: make these a Vec of Box<dyn Force>
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
        // Update systems
        for system in &mut self.systems {
            // TODO time each system (how to report system name? use index?)
            system.update(&mut self.particles, dt);
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

            // TODO use custom integrator?
            // Update particle
            particle.velocity += forces * dt / particle.mass;
            particle.position += particle.velocity * dt;
        }
    }
}

pub trait SimulationRunner {
    fn step(&mut self);
    fn get_simulation_mut(&mut self) -> &mut Simulation;
    fn get_simulation(&self) -> &Simulation;
}

pub struct ContinuousSimulationRunner {
    simulation: Simulation,
    dt: f32,
}

impl ContinuousSimulationRunner {
    pub fn new(simulation: Simulation, dt: f32) -> Self {
        Self { simulation, dt }
    }
}

impl SimulationRunner for ContinuousSimulationRunner {
    fn step(&mut self) {
        self.simulation.step(self.dt);
    }

    fn get_simulation_mut(&mut self) -> &mut Simulation {
        &mut self.simulation
    }

    fn get_simulation(&self) -> &Simulation {
        &self.simulation
    }
}
