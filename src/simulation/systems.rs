use nalgebra::Vector2;
use rayon::prelude::*;

use super::{
    areas::Area,
    integrator::Integrator,
    particles::{ParticleFactory, Particles},
    types::{Force as TypeForce, Scalar, Time},
};

pub trait System {
    fn update(&mut self, particles: &mut Particles, dt: Time);

    fn get_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
}

pub struct ConstantConsumer {
    pub area: Box<dyn Area>,
    pub rate: Scalar,
}

impl ConstantConsumer {
    pub fn new(area: Box<dyn Area>, rate: Scalar) -> Self {
        Self { area, rate }
    }
}

// Handle the case where the rate is not an integer
// Making the rate smooth & accurate across steps
fn smooth_rate(rate: Scalar, dt: Time) -> usize {
    let n = rate * dt;
    let mut quotient = n as usize;

    // Remainder
    // TODO make deterministic (keep track of remainder)
    if rand::random::<Scalar>() < n - quotient as Scalar {
        quotient += 1;
    }

    quotient
}

impl System for ConstantConsumer {
    fn update(&mut self, particles: &mut Particles, dt: Time) {
        let mut quotient = smooth_rate(self.rate, dt);

        let mut to_remove = Vec::new();
        self.area.contains(&particles.positions, &mut to_remove);

        // TODO parallelize????????
        for i in to_remove.iter().rev() {
            particles.swap_remove(*i);
            quotient -= 1;
            if quotient == 0 {
                break;
            }
        }
    }
}

pub struct ConstantEmitter {
    pub p_factory: Box<dyn ParticleFactory>,
    pub rate: Scalar,
}

impl ConstantEmitter {
    pub fn new(p_factory: Box<dyn ParticleFactory>, rate: Scalar) -> Self {
        Self { p_factory, rate }
    }
}

impl System for ConstantEmitter {
    fn update(&mut self, particles: &mut Particles, dt: Time) {
        let quotient = smooth_rate(self.rate, dt);
        self.p_factory.create(quotient, particles);
    }
}

pub struct Wall {
    pub x_min: Scalar,
    pub y_min: Scalar,
    pub x_max: Scalar,
    pub y_max: Scalar,
    pub restitution: Scalar,
}

impl System for Wall {
    fn update(&mut self, particles: &mut Particles, _dt: Time) {
        particles
            .positions
            .par_iter_mut()
            .zip(particles.velocities.par_iter_mut())
            .for_each(|(position, velocity)| {
                if position.x < self.x_min {
                    position.x = self.x_min;
                    velocity.x = -velocity.x * self.restitution;
                } else if position.x > self.x_max {
                    position.x = self.x_max;
                    velocity.x = -velocity.x * self.restitution;
                }

                if position.y < self.y_min {
                    position.y = self.y_min;
                    velocity.y = -velocity.y * self.restitution;
                } else if position.y > self.y_max {
                    position.y = self.y_max;
                    velocity.y = -velocity.y * self.restitution;
                }
            });
    }
}

pub struct Loop {
    pub x_min: Scalar,
    pub y_min: Scalar,
    pub x_max: Scalar,
    pub y_max: Scalar,
}

impl System for Loop {
    fn update(&mut self, particles: &mut Particles, _dt: Time) {
        particles.positions.par_iter_mut().for_each(|position| {
            if position.x < self.x_min {
                position.x = self.x_max;
            } else if position.x > self.x_max {
                position.x = self.x_min;
            }

            if position.y < self.y_min {
                position.y = self.y_max;
            } else if position.y > self.y_max {
                position.y = self.y_min;
            }
        });
    }
}

pub struct Void {
    pub area: Box<dyn Area>,
}

impl System for Void {
    fn update(&mut self, particles: &mut Particles, _dt: Time) {
        let mut to_remove = Vec::new();
        self.area.contains(&particles.positions, &mut to_remove);

        // TODO parallelize????????
        for i in to_remove {
            particles.swap_remove(i);
        }
    }
}

pub trait Force {
    fn apply(&self, particles: &Particles, forces: &mut Vec<TypeForce>);
}

pub struct Physics {
    forces: Vec<Box<dyn Force>>,
    integrator: Box<dyn Integrator<Vector2<Scalar>>>,
    forces_buffer: Vec<TypeForce>,
}

impl Physics {
    pub fn new(
        forces: Vec<Box<dyn Force>>,
        integrator: Box<dyn Integrator<Vector2<Scalar>>>,
    ) -> Self {
        Self {
            forces,
            integrator,
            forces_buffer: Vec::new(),
        }
    }
}

impl System for Physics {
    fn update(&mut self, particles: &mut Particles, dt: Time) {
        self.forces_buffer.clear();
        self.forces_buffer.resize(particles.len(), Vector2::zeros());

        for force in self.forces.iter() {
            force.apply(particles, &mut self.forces_buffer);
        }

        self.integrator
            .integrate_vec(&self.forces_buffer, &mut particles.velocities, dt);
    }
}

pub struct VelocityIntegrator {
    pub integrator: Box<dyn Integrator<Vector2<Scalar>>>,
}

impl VelocityIntegrator {
    pub fn new(integrator: Box<dyn Integrator<Vector2<Scalar>>>) -> Self {
        Self { integrator }
    }
}

impl System for VelocityIntegrator {
    fn update(&mut self, particles: &mut Particles, dt: Time) {
        self.integrator
            .integrate_vec(&particles.velocities, &mut particles.positions, dt);
    }
}
