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

pub struct Gravity {
    pub coef: Scalar,
    pub epsilon: Scalar,
}

impl Gravity {
    pub fn new(coef: Scalar, epsilon: Scalar) -> Self {
        Self { coef, epsilon }
    }
}

impl Force for Gravity {
    fn apply(&self, particles: &Particles, forces: &mut Vec<ForceType>) {
        rayon::scope(|s| {
            let num_threads = rayon::current_num_threads();
            let particles_per_thread = (particles.positions.len() + num_threads - 1) / num_threads;

            let force_arc = std::sync::Arc::new(std::sync::Mutex::new(forces));

            for thread_id in 0..num_threads {
                let force_clone = force_arc.clone();
                let start = thread_id * particles_per_thread;
                let end = std::cmp::min(start + particles_per_thread, particles.positions.len());

                s.spawn(move |_| {
                    let mut local_forces = vec![ForceType::zeros(); particles.positions.len()];
                    for i in start..end {
                        for j in (i + 1)..particles.positions.len() {
                            let distance_v = particles.positions[i] - particles.positions[j];
                            let distance = distance_v.norm();

                            if distance < self.epsilon {
                                continue;
                            }

                            let force = self.coef * particles.masses[i] * particles.masses[j]
                                / distance.powi(3)
                                * distance_v;

                            local_forces[i] -= force;
                            local_forces[j] += force;
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

pub struct Drag {
    pub coef: Scalar,
    pub distance: Scalar,
}

impl Drag {
    pub fn new(coef: Scalar, distance: Scalar) -> Self {
        Self { coef, distance }
    }
}

impl Force for Drag {
    fn apply(&self, particles: &Particles, forces: &mut Vec<ForceType>) {
        rayon::scope(|s| {
            let num_threads = rayon::current_num_threads();
            let particles_per_thread = (particles.positions.len() + num_threads - 1) / num_threads;

            let force_arc = std::sync::Arc::new(std::sync::Mutex::new(forces));

            for thread_id in 0..num_threads {
                let force_clone = force_arc.clone();
                let start = thread_id * particles_per_thread;
                let end = std::cmp::min(start + particles_per_thread, particles.positions.len());

                s.spawn(move |_| {
                    let mut local_forces = vec![ForceType::zeros(); particles.positions.len()];
                    for i in start..end {
                        for j in (i + 1)..particles.positions.len() {
                            let distance_v = particles.positions[i] - particles.positions[j];

                            let distance = distance_v.norm();

                            if distance > self.distance || distance == 0.0 {
                                continue;
                            }

                            // Linear interpolation between 0 (self.distance) and 1 (0)
                            let dist_coef = 1.0 - distance / self.distance;
                            let velocity_diff = particles.velocities[i] - particles.velocities[j];
                            let force = (-self.coef * dist_coef) * velocity_diff;

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

pub struct Repulsion {
    pub coef: Scalar,
    pub epsilon: Scalar,
}

impl Repulsion {
    pub fn new(coef: Scalar, epsilon: Scalar) -> Self {
        Self { coef, epsilon }
    }
}

impl Force for Repulsion {
    fn apply(&self, particles: &Particles, forces: &mut Vec<ForceType>) {
        rayon::scope(|s| {
            let num_threads = rayon::current_num_threads();
            let particles_per_thread = (particles.positions.len() + num_threads - 1) / num_threads;

            let force_arc = std::sync::Arc::new(std::sync::Mutex::new(forces));

            for thread_id in 0..num_threads {
                let force_clone = force_arc.clone();
                let start = thread_id * particles_per_thread;
                let end = std::cmp::min(start + particles_per_thread, particles.positions.len());

                s.spawn(move |_| {
                    let mut local_forces = vec![ForceType::zeros(); particles.positions.len()];
                    for i in start..end {
                        for j in (i + 1)..particles.positions.len() {
                            let distance_v = particles.positions[i] - particles.positions[j];
                            let distance = distance_v.norm();

                            if distance < self.epsilon {
                                continue;
                            }

                            let force = self.coef * distance_v / distance.powi(4);

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
