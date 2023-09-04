use rayon::prelude::*;

use crate::{
    particles::Particles,
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
        particles
            .masses
            .par_iter()
            .zip(forces.par_iter_mut())
            .for_each(|(mass, force)| {
                *force += *mass * self.acceleration;
            });
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
        particles
            .velocities
            .par_iter()
            .zip(forces.par_iter_mut())
            .for_each(|(velocity, force)| {
                *force -= self.coef * (velocity - &self.velocity);
            });
    }
}
