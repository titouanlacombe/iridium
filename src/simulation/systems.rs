use nalgebra::Vector2;
use rayon::prelude::*;

use super::{
    areas::Area,
    color::Color,
    forces::Force,
    integrator::Integrator,
    particles::{ParticleFactory, Particles},
    types::{Force as TypeForce, Scalar, Time},
};
use crate::utils::smooth_rate::SmoothRate;

pub trait System {
    fn update(&mut self, particles: &mut Particles, dt: Time);

    fn get_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
}

pub struct ConstantConsumer {
    area: Box<dyn Area>,
    rate: SmoothRate,
}

impl ConstantConsumer {
    pub fn new(area: Box<dyn Area>, rate: Scalar) -> Self {
        Self {
            area,
            rate: SmoothRate::new(rate),
        }
    }
}

impl System for ConstantConsumer {
    fn update(&mut self, particles: &mut Particles, dt: Time) {
        let mut quotient = self.rate.get(dt);

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
    p_factory: Box<dyn ParticleFactory>,
    rate: SmoothRate,
}

impl ConstantEmitter {
    pub fn new(p_factory: Box<dyn ParticleFactory>, rate: Scalar) -> Self {
        Self {
            p_factory,
            rate: SmoothRate::new(rate),
        }
    }
}

impl System for ConstantEmitter {
    fn update(&mut self, particles: &mut Particles, dt: Time) {
        let quotient = self.rate.get(dt);
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

        for force in self.forces.iter_mut() {
            force.apply(particles, &mut self.forces_buffer);
        }

        // Scale forces by mass to get acceleration
        self.forces_buffer
            .par_iter_mut()
            .zip(particles.masses.par_iter())
            .for_each(|(force, mass)| {
                *force /= *mass;
            });

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

pub struct ColorWheel {
    pub speed: Scalar,
}

impl System for ColorWheel {
    fn update(&mut self, particles: &mut Particles, dt: Time) {
        particles.colors.par_iter_mut().for_each(|color| {
            let (h, s, v, a) = color.to_hsva();
            let h = (h + self.speed * dt) % 360.0;
            *color = Color::from_hsva(h, s, v, a);
        });
    }
}
