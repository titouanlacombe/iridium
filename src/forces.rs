use nalgebra::Vector2;

use crate::{particle::Particles, systems::Force};

pub struct UniformGravity {
    pub acceleration: Vector2<f32>,
}

impl UniformGravity {
    pub fn new(acceleration: Vector2<f32>) -> Self {
        Self { acceleration }
    }
}

impl Force for UniformGravity {
    fn apply(&self, particles: &Particles, forces: &mut Vec<Vector2<f32>>) {
        for (i, mass) in particles.masses.iter().enumerate() {
            forces[i] += self.acceleration * *mass;
        }
    }
}

pub struct UniformDrag {
    pub coef: f32,
}

impl UniformDrag {
    pub fn new(coef: f32) -> Self {
        Self { coef }
    }
}

impl Force for UniformDrag {
    fn apply(&self, particles: &Particles, forces: &mut Vec<Vector2<f32>>) {
        for (i, velocity) in particles.velocities.iter().enumerate() {
            forces[i] -= *velocity * -self.coef;
        }
    }
}
