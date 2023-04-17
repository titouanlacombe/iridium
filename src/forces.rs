use crate::{
    particle::Particles,
    systems::Force as PhysForce,
    types::{Acceleration, Force, Scalar, Velocity},
};

pub struct UniformGravity {
    pub acceleration: Acceleration,
}

impl UniformGravity {
    pub fn new(acceleration: Acceleration) -> Self {
        Self { acceleration }
    }
}

impl PhysForce for UniformGravity {
    fn apply(&self, particles: &Particles, forces: &mut Vec<Force>) {
        for (i, mass) in particles.masses.iter().enumerate() {
            forces[i] += self.acceleration * *mass;
        }
    }
}

pub struct UniformDrag {
    pub coef: Scalar,
    pub velocity: Velocity,
}

impl UniformDrag {
    pub fn new(coef: Scalar, velocity: Velocity) -> Self {
        Self { coef, velocity }
    }
}

impl PhysForce for UniformDrag {
    fn apply(&self, particles: &Particles, forces: &mut Vec<Force>) {
        for (i, velocity) in particles.velocities.iter().enumerate() {
            forces[i] -= self.coef * (velocity - &self.velocity);
        }
    }
}
