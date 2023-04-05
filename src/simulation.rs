use nalgebra::Vector2;

use crate::{
    forces::{UniformDrag, UniformGravity},
    particle::Particle,
    updatables::Updatable,
};

pub enum LimitCond {
    // infinite space
    // no effect
    None,

    // wall at position, with size and restitution
    // will bounce objects
    Wall(f32, f32, f32, f32, f32),

    // loop at position, with size
    // object will be teleported to the other side
    Loop(f32, f32, f32, f32),

    // void at position, with size
    // object will be destroyed on exit
    Void(f32, f32, f32, f32),
}

fn limit_update(limit: &LimitCond, i: usize, particle: &mut Particle, to_remove: &mut Vec<usize>) {
    match limit {
        LimitCond::None => {}
        LimitCond::Wall(x_min, y_min, x_max, y_max, restitution) => {
            let p_pos = &mut particle.position;
            let p_vel = &mut particle.velocity;

            if p_pos.x < *x_min {
                p_pos.x = *x_min;
                p_vel.x = -p_vel.x * restitution;
            } else if p_pos.x > *x_max {
                p_pos.x = *x_max;
                p_vel.x = -p_vel.x * restitution;
            }

            if p_pos.y < *y_min {
                p_pos.y = *y_min;
                p_vel.y = -p_vel.y * restitution;
            } else if p_pos.y > *y_max {
                p_pos.y = *y_max;
                p_vel.y = -p_vel.y * restitution;
            }
        }
        LimitCond::Loop(x_min, y_min, x_max, y_max) => {
            let p_pos = &mut particle.position;

            if p_pos.x < *x_min {
                p_pos.x = *x_max;
            } else if p_pos.x > *x_max {
                p_pos.x = *x_min;
            }

            if p_pos.y < *y_min {
                p_pos.y = *y_max;
            } else if p_pos.y > *y_max {
                p_pos.y = *y_min;
            }
        }
        LimitCond::Void(x_min, y_min, x_max, y_max) => {
            let p_pos = &mut particle.position;

            if p_pos.x < *x_min || p_pos.x > *x_max || p_pos.y < *y_min || p_pos.y > *y_max {
                to_remove.push(i);
            }
        }
    }
}

pub struct Simulation {
    pub particles: Vec<Particle>,
    updatables: Vec<Box<dyn Updatable>>,

    // TODO: make these a Vec of Box<dyn Force>
    uniform_gravity: Option<UniformGravity>,
    uniform_drag: Option<UniformDrag>,

    limit: LimitCond,
    // TODO add sim event handler
}

impl Simulation {
    pub fn new(
        particles: Vec<Particle>,
        updatables: Vec<Box<dyn Updatable>>,
        uniform_gravity: Option<UniformGravity>,
        uniform_drag: Option<UniformDrag>,
        limit: LimitCond,
    ) -> Self {
        Self {
            particles,
            updatables,
            uniform_gravity,
            uniform_drag,
            limit,
        }
    }

    pub fn step(&mut self, dt: f32) {
        // Update updatables
        for updatable in &mut self.updatables {
            updatable.update(&mut self.particles, dt);
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

        // Check limits
        let mut to_remove: Vec<usize> = Vec::new();
        for (i, particle) in self.particles.iter_mut().enumerate() {
            limit_update(&self.limit, i, particle, &mut to_remove)
        }
        for i in to_remove {
            self.particles.swap_remove(i);
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
