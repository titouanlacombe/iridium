use rayon::prelude::*;

use super::{
    particles::Particles,
    types::{mask_to_01, masked, repulsion_inv_pow, Acceleration, Force as ForceType, Mass, Position, Scalar, Simd, SimdVec, Velocity},
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
// force types.
// The inner loop is vectorized: SIMD lanes process consecutive j's for a fixed i.
// `kernel(i, jx, jy, jvx, jvy, jm) -> (fx, fy)` returns the force the j-batch
// (one particle per lane) exerts on particle i; the pair is applied with opposite
// signs (+ on i, - on j, Newton's third law).
fn apply_pairwise_simd<F>(n: usize, particles: &Particles, forces: &mut Vec<ForceType>, kernel: F)
where
    F: Fn(usize, Simd, Simd, Simd, Simd, Simd) -> (Simd, Simd) + Sync,
{
    let num_threads = rayon::current_num_threads();
    let particles_per_thread = (n + num_threads - 1) / num_threads;
    let kernel = &kernel;

    let local_forces: Vec<Vec<ForceType>> = (0..num_threads)
        .into_par_iter()
        .map(|thread_id| {
            let start = thread_id * particles_per_thread;
            let end = (start + particles_per_thread).min(n);

            let mut local = vec![ForceType::zeros(); n];
            for i in start..end {
                let mut j = i + 1;
                while j < n {
                    // Load the j-batch. Invalid tail lanes are skipped at extraction.
                    let mut jx = [0.0; 8];
                    let mut jy = [0.0; 8];
                    let mut jvx = [0.0; 8];
                    let mut jvy = [0.0; 8];
                    let mut jm = [0.0; 8];
                    for k in 0..Simd::LANES {
                        if j + k < n {
                            let p = particles.positions[j + k];
                            let v = particles.velocities[j + k];
                            jx[k] = p.x;
                            jy[k] = p.y;
                            jvx[k] = v.x;
                            jvy[k] = v.y;
                            jm[k] = particles.masses[j + k];
                        }
                    }
                    let (fx, fy) = kernel(
                        i,
                        Simd::from_8(&jx),
                        Simd::from_8(&jy),
                        Simd::from_8(&jvx),
                        Simd::from_8(&jvy),
                        Simd::from_8(&jm),
                    );

                    let mut fx_arr = [0.0; 8];
                    let mut fy_arr = [0.0; 8];
                    fx.write_8(&mut fx_arr);
                    fy.write_8(&mut fy_arr);

                    let mut sum_x = 0.0;
                    let mut sum_y = 0.0;
                    for k in 0..Simd::LANES {
                        if j + k < n {
                            sum_x += fx_arr[k];
                            sum_y += fy_arr[k];
                            local[j + k] -= ForceType::new(fx_arr[k], fy_arr[k]);
                        }
                    }
                    local[i] += ForceType::new(sum_x, sum_y);
                    j += Simd::LANES;
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

        apply_pairwise_simd(particles.len(), particles, forces, |i, jx, jy, _jvx, _jvy, jm| {
            let p = particles.positions[i];
            let mi = particles.masses[i];
            let dx = Simd::splat(p.x) - jx;
            let dy = Simd::splat(p.y) - jy;
            let r2 = dx * dx + dy * dy;
            let r = r2.sqrt();
            // Validity mask must select, not multiply: near-zero r gives inf/NaN
            // (0/0 at exact duplicates), and inf * 0.0 is NaN.
            let g_valid = mask_to_01(r.mask_ge(Simd::splat(immut_self.epsilon)));
            let r3 = r * r2;
            let g_scale = -Simd::splat(immut_self.coef) * Simd::splat(mi) * jm / r3;
            (
                masked(g_valid, g_scale * dx),
                masked(g_valid, g_scale * dy),
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

        apply_pairwise_simd(particles.len(), particles, forces, |i, jx, jy, jvx, jvy, _jm| {
            let p = particles.positions[i];
            let v = particles.velocities[i];
            let dx = Simd::splat(p.x) - jx;
            let dy = Simd::splat(p.y) - jy;
            let r2 = dx * dx + dy * dy;
            let r = r2.sqrt();
            let drag_valid = mask_to_01(r.mask_le(Simd::splat(immut_self.distance)))
                * mask_to_01(r.mask_gt(Simd::splat(0.0)));
            // Mask the ratio first: distance == 0 would divide by zero
            let dist_ratio = masked(drag_valid, r / Simd::splat(immut_self.distance));
            let dist_coef = Simd::splat(1.0) - dist_ratio * dist_ratio;
            let dvx = Simd::splat(v.x) - jvx;
            let dvy = Simd::splat(v.y) - jvy;
            let drag_scale = -Simd::splat(immut_self.coef) * dist_coef;
            (
                masked(drag_valid, drag_scale * dvx),
                masked(drag_valid, drag_scale * dvy),
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

        apply_pairwise_simd(particles.len(), particles, forces, |i, jx, jy, _jvx, _jvy, _jm| {
            let p = particles.positions[i];
            let dx = Simd::splat(p.x) - jx;
            let dy = Simd::splat(p.y) - jy;
            let r2 = dx * dx + dy * dy;
            let r = r2.sqrt();
            // Validity mask must select, not multiply: r^(-power) overflows to inf
            // for tiny r in f32, and inf * 0.0 is NaN.
            let rep_valid = mask_to_01(r.mask_ge(Simd::splat(immut_self.epsilon)));
            let rep_scale = Simd::splat(immut_self.coef) * repulsion_inv_pow(immut_self.power, r);
            (
                masked(rep_valid, rep_scale * dx),
                masked(rep_valid, rep_scale * dy),
            )
        });
    }
}
