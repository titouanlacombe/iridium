use rayon::prelude::*;

use super::{
    particles::Particles,
    types::{Acceleration, Force as ForceType, Mass, Position, Scalar, Velocity},
};

pub trait Force {
    fn apply(&mut self, particles: &Particles, forces: &mut Vec<ForceType>);

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

// Shared machinery for O(n^2) pairwise forces: each rayon thread computes its own
// local buffer (no shared state), then merges in a fixed per-index order so results
// are deterministic. Accumulates: forces may already hold contributions from other
// force types. `calc(i, j)` returns the force that particle i exerts on j.
fn apply_pairwise<F>(n: usize, forces: &mut Vec<ForceType>, calc: F)
where
    F: Fn(usize, usize) -> ForceType + Sync,
{
    let num_threads = rayon::current_num_threads();
    let particles_per_thread = (n + num_threads - 1) / num_threads;
    let calc = &calc;

    let local_forces: Vec<Vec<ForceType>> = (0..num_threads)
        .into_par_iter()
        .map(|thread_id| {
            let start = thread_id * particles_per_thread;
            let end = (start + particles_per_thread).min(n);

            let mut local = vec![ForceType::zeros(); n];
            for i in start..end {
                for j in (i + 1)..n {
                    let force = calc(i, j);

                    local[i] += force;
                    local[j] -= force;
                }
            }
            local
        })
        .collect::<Vec<_>>();

    forces.par_iter_mut().enumerate().for_each(|(i, force)| {
        *force += local_forces.iter().map(|local| local[i]).sum::<ForceType>();
    });
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
    fn apply(&mut self, particles: &Particles, forces: &mut Vec<ForceType>) {
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
    fn apply(&mut self, particles: &Particles, forces: &mut Vec<ForceType>) {
        particles
            .velocities
            .par_iter()
            .zip(forces.par_iter_mut())
            .for_each(|(velocity, force)| {
                *force -= self.coef * (velocity - &self.velocity);
            });
    }
}

#[derive(Clone)]
pub struct Gravity {
    pub coef: Scalar,
    pub epsilon: Scalar,
}

impl Gravity {
    pub fn new(coef: Scalar, epsilon: Scalar) -> Self {
        Self { coef, epsilon }
    }

    #[inline]
    pub fn calc_force(
        &self,
        pos1: Position,
        pos2: Position,
        mass1: Mass,
        mass2: Mass,
    ) -> ForceType {
        let distance_v = pos1 - pos2;
        let distance = distance_v.norm();

        if distance < self.epsilon {
            return ForceType::zeros();
        }

        -self.coef * distance_v * mass1 * mass2 / distance.powi(3)
    }
}

impl Force for Gravity {
    fn apply(&mut self, particles: &Particles, forces: &mut Vec<ForceType>) {
        let immut_self = &*self;

        apply_pairwise(particles.len(), forces, |i, j| {
            immut_self.calc_force(
                particles.positions[i],
                particles.positions[j],
                particles.masses[i],
                particles.masses[j],
            )
        });
    }
}

#[derive(Clone)]
pub struct Drag {
    pub coef: Scalar,
    pub distance: Scalar,
}

impl Drag {
    pub fn new(coef: Scalar, distance: Scalar) -> Self {
        Self { coef, distance }
    }

    #[inline]
    pub fn calc_force(
        &self,
        pos1: Position,
        pos2: Position,
        vel1: Velocity,
        vel2: Velocity,
    ) -> ForceType {
        let distance = (pos1 - pos2).norm();

        if distance > self.distance || distance == 0.0 {
            return ForceType::zeros();
        }

        // Quadratic interpolation between 0 (f_distance) and 1 (0)
        let dist_coef = 1.0 - (distance / self.distance).powi(2);
        let velocity_diff = vel1 - vel2;
        (-self.coef * dist_coef) * velocity_diff
    }
}

impl Force for Drag {
    fn apply(&mut self, particles: &Particles, forces: &mut Vec<ForceType>) {
        let immut_self = &*self;

        apply_pairwise(particles.len(), forces, |i, j| {
            immut_self.calc_force(
                particles.positions[i],
                particles.positions[j],
                particles.velocities[i],
                particles.velocities[j],
            )
        });
    }
}

#[derive(Clone)]
pub struct Repulsion {
    pub coef: Scalar,
    pub power: i32,
    pub epsilon: Scalar,
}

impl Repulsion {
    pub fn new(coef: Scalar, power: i32, epsilon: Scalar) -> Self {
        Self {
            coef,
            power,
            epsilon,
        }
    }

    #[inline]
    pub fn calc_force(&self, pos1: Position, pos2: Position) -> ForceType {
        let distance_v = pos1 - pos2;
        let distance = distance_v.norm();

        if distance < self.epsilon {
            return ForceType::zeros();
        }

        self.coef * distance_v / distance.powi(self.power)
    }
}

impl Force for Repulsion {
    fn apply(&mut self, particles: &Particles, forces: &mut Vec<ForceType>) {
        let immut_self = &*self;

        apply_pairwise(particles.len(), forces, |i, j| {
            immut_self.calc_force(particles.positions[i], particles.positions[j])
        });
    }
}
