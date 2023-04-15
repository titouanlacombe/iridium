use nalgebra::Vector2;

use crate::{
    areas::Area,
    particle::{ParticleFactory, Particles},
};

pub trait System {
    fn update(&mut self, particles: &mut Particles, dt: f32);
}

pub struct ConstantConsumer {
    pub area: Box<dyn Area>,
    pub rate: f32,
}

impl ConstantConsumer {
    pub fn new(area: Box<dyn Area>, rate: f32) -> Self {
        Self { area, rate }
    }
}

// Handle the case where the rate is not an integer
// Making the rate smooth & accurate across steps
fn smooth_rate(rate: f32, dt: f32) -> usize {
    let n = rate * dt;
    let mut quotient = n as usize;

    // Remainder
    if rand::random::<f32>() < n - quotient as f32 {
        quotient += 1;
    }

    quotient
}

impl System for ConstantConsumer {
    fn update(&mut self, particles: &mut Particles, dt: f32) {
        let mut quotient = smooth_rate(self.rate, dt);

        let mut to_remove = Vec::new();
        self.area.contains(&particles.positions, &mut to_remove);

        for i in to_remove {
            particles.swap_remove(i);
            quotient -= 1;
            if quotient == 0 {
                break;
            }
        }
    }
}

pub struct ConstantEmitter {
    pub p_factory: Box<dyn ParticleFactory>,
    pub rate: f32,
}

impl ConstantEmitter {
    pub fn new(p_factory: Box<dyn ParticleFactory>, rate: f32) -> Self {
        Self { p_factory, rate }
    }
}

impl System for ConstantEmitter {
    fn update(&mut self, particles: &mut Particles, dt: f32) {
        let quotient = smooth_rate(self.rate, dt);
        self.p_factory.create(quotient, particles);
    }
}

pub struct Wall {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
    pub restitution: f32,
}

impl System for Wall {
    fn update(&mut self, particles: &mut Particles, _dt: f32) {
        for (position, velocity, _mass) in particles.iter_mut() {
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
        }
    }
}

pub struct Loop {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

impl System for Loop {
    fn update(&mut self, particles: &mut Particles, _dt: f32) {
        for position in particles.positions.iter_mut() {
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
        }
    }
}

pub struct Void {
    pub area: Box<dyn Area>,
}

impl System for Void {
    fn update(&mut self, particles: &mut Particles, _dt: f32) {
        let mut to_remove = Vec::new();
        self.area.contains(&particles.positions, &mut to_remove);

        for i in to_remove {
            particles.swap_remove(i);
        }
    }
}

pub trait Force {
    fn apply(&self, particles: &Particles, forces: &mut Vec<Vector2<f32>>);
}

pub trait Integrator {
    fn integrate(&self, particles: &mut Particles, forces: &Vec<Vector2<f32>>, dt: f32);
}

pub struct Physics {
    forces: Vec<Box<dyn Force>>,
    integrator: Box<dyn Integrator>,
    forces_buffer: Vec<Vector2<f32>>,
}

impl Physics {
    pub fn new(forces: Vec<Box<dyn Force>>, integrator: Box<dyn Integrator>) -> Self {
        Self {
            forces,
            integrator,
            forces_buffer: Vec::new(),
        }
    }
}

impl System for Physics {
    fn update(&mut self, particles: &mut Particles, dt: f32) {
        self.forces_buffer.clear();
        self.forces_buffer.resize(particles.len(), Vector2::zeros());

        for force in self.forces.iter() {
            force.apply(particles, &mut self.forces_buffer);
        }

        self.integrator
            .integrate(particles, &self.forces_buffer, dt);
    }
}

pub struct GaussianIntegrator;

impl GaussianIntegrator {
    pub fn new() -> Self {
        Self
    }
}

impl Integrator for GaussianIntegrator {
    fn integrate(&self, particles: &mut Particles, forces: &Vec<Vector2<f32>>, dt: f32) {
        for (i, particle) in particles.iter_mut().enumerate() {
            let (position, velocity, mass) = particle;
            *velocity += ((*forces)[i] / *mass) * dt;
            *position += *velocity * dt;
        }
    }
}
