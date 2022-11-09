use nalgebra::Vector2;

use crate::particle::Particle;

pub struct UniformGravity {
    pub acceleration: Vector2<f32>,
}

impl UniformGravity {
    pub fn new(acceleration: Vector2<f32>) -> Self {
        Self { acceleration }
    }

    pub fn apply(&self, particle: &mut Particle, forces: &mut Vector2<f32>) {
        *forces += self.acceleration * particle.mass;
    }
}

pub struct UniformDrag {
    pub coef: f32,
}

impl UniformDrag {
    pub fn new(drag: f32) -> Self {
        Self { coef: 1. - drag }
    }

    pub fn apply(&self, particle: &mut Particle, forces: &mut Vector2<f32>) {
        *forces += self.coef * particle.velocity;
    }
}
