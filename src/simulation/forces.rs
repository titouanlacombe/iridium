use rayon::prelude::*;

use super::{
    particles::Particles,
    types::{Acceleration, Force as ForceType, Scalar, Velocity},
};

pub trait Force {
    fn apply(&self, particles: &Particles, forces: &mut Vec<ForceType>);
}

pub struct UniformGravity {
    pub acceleration: Acceleration,
}

impl UniformGravity {
    pub fn new(acceleration: Acceleration) -> Self {
        Self { acceleration }
    }
}

impl Force for UniformGravity {
    fn apply(&self, particles: &Particles, forces: &mut Vec<ForceType>) {
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

impl Force for UniformDrag {
    fn apply(&self, particles: &Particles, forces: &mut Vec<ForceType>) {
        particles
            .velocities
            .par_iter()
            .zip(forces.par_iter_mut())
            .for_each(|(velocity, force)| {
                *force -= self.coef * (velocity - &self.velocity);
            });
    }
}
