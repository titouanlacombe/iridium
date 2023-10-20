use rayon::prelude::*;

use super::{
    particles::Particles,
    types::{Acceleration, Force as ForceType, Mass, Position, Scalar, Velocity},
};

pub trait Force {
    fn apply(&mut self, particles: &Particles, forces: &mut Vec<ForceType>);
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
        // TODO refactor: first create a vec of all the combinations of particles
        // par iter on this vec and collect the results in a vec of forces indexed by the index of the particle
        // then add the results to the forces vec

        rayon::scope(|s| {
            let num_threads = rayon::current_num_threads();
            let particles_per_thread = (particles.positions.len() + num_threads - 1) / num_threads;

            let force_arc = std::sync::Arc::new(std::sync::Mutex::new(forces));

            for thread_id in 0..num_threads {
                let force_clone = force_arc.clone();
                let start = thread_id * particles_per_thread;
                let end = std::cmp::min(start + particles_per_thread, particles.positions.len());
                let immut_self = &*self;

                s.spawn(move |_| {
                    let mut local_forces = vec![ForceType::zeros(); particles.positions.len()];
                    for i in start..end {
                        for j in (i + 1)..particles.positions.len() {
                            let force = immut_self.calc_force(
                                particles.positions[i],
                                particles.positions[j],
                                particles.masses[i],
                                particles.masses[j],
                            );

                            local_forces[i] += force;
                            local_forces[j] -= force;
                        }
                    }
                    let mut global_forces = force_clone.lock().unwrap();
                    for (i, force) in local_forces.into_iter().enumerate() {
                        global_forces[i] += force;
                    }
                });
            }
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
        rayon::scope(|s| {
            let num_threads = rayon::current_num_threads();
            let particles_per_thread = (particles.positions.len() + num_threads - 1) / num_threads;

            let force_arc = std::sync::Arc::new(std::sync::Mutex::new(forces));

            for thread_id in 0..num_threads {
                let force_clone = force_arc.clone();
                let start = thread_id * particles_per_thread;
                let end = std::cmp::min(start + particles_per_thread, particles.positions.len());
                let immut_self = &*self;

                s.spawn(move |_| {
                    let mut local_forces = vec![ForceType::zeros(); particles.positions.len()];
                    for i in start..end {
                        for j in (i + 1)..particles.positions.len() {
                            let force = immut_self.calc_force(
                                particles.positions[i],
                                particles.positions[j],
                                particles.velocities[i],
                                particles.velocities[j],
                            );

                            local_forces[i] += force;
                            local_forces[j] -= force;
                        }
                    }
                    let mut global_forces = force_clone.lock().unwrap();
                    for (i, force) in local_forces.into_iter().enumerate() {
                        global_forces[i] += force;
                    }
                });
            }
        });
    }
}

#[derive(Clone)]
pub struct Repulsion {
    pub coef: Scalar,
    pub epsilon: Scalar,
}

impl Repulsion {
    pub fn new(coef: Scalar, epsilon: Scalar) -> Self {
        Self { coef, epsilon }
    }

    #[inline]
    pub fn calc_force(&self, pos1: Position, pos2: Position) -> ForceType {
        let distance_v = pos1 - pos2;
        let distance = distance_v.norm();

        if distance < self.epsilon {
            return ForceType::zeros();
        }

        self.coef * distance_v / distance.powi(4)
    }
}

impl Force for Repulsion {
    fn apply(&mut self, particles: &Particles, forces: &mut Vec<ForceType>) {
        rayon::scope(|s| {
            let num_threads = rayon::current_num_threads();
            let particles_per_thread = (particles.positions.len() + num_threads - 1) / num_threads;

            let force_arc = std::sync::Arc::new(std::sync::Mutex::new(forces));

            for thread_id in 0..num_threads {
                let force_clone = force_arc.clone();
                let start = thread_id * particles_per_thread;
                let end = std::cmp::min(start + particles_per_thread, particles.positions.len());
                let immut_self = &*self;

                s.spawn(move |_| {
                    let mut local_forces = vec![ForceType::zeros(); particles.positions.len()];
                    for i in start..end {
                        for j in (i + 1)..particles.positions.len() {
                            let force = immut_self
                                .calc_force(particles.positions[i], particles.positions[j]);

                            local_forces[i] += force;
                            local_forces[j] -= force;
                        }
                    }
                    let mut global_forces = force_clone.lock().unwrap();
                    for (i, force) in local_forces.into_iter().enumerate() {
                        global_forces[i] += force;
                    }
                });
            }
        });
    }
}
