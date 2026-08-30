use rayon::prelude::*;

use super::{
    color::Color,
    particles::Particles,
    systems::System,
    types::{Mass, Position, Scalar, Time, Velocity},
};

// Spread 21 bits of v into the even bit positions of a 42-bit value (Morton interleave).
fn spread_bits(v: u64) -> u64 {
    let mut x = v & 0x1FFFFF;
    x = (x | x << 32) & 0x1F00000000FFFF;
    x = (x | x << 16) & 0x1F0000FF0000FF;
    x = (x | x << 8) & 0x100F00F00F00F00F;
    x = (x | x << 4) & 0x10C30C30C30C30C3;
    x = (x | x << 2) & 0x1249249249249249;
    x
}

// 2D Morton (Z-order) code: interleaves the x and y bits (21 bits per axis).
fn morton_code(x: u32, y: u32) -> u64 {
    spread_bits(x as u64) | (spread_bits(y as u64) << 1)
}

// Sorts the particle SoA along a 2D Morton curve so that spatially close particles
// have close indices. Batched Barnes-Hut traversals rely on this coherence.
// Must run before Physics when using the quadtree forces.
pub struct MortonSort {
    codes: Vec<u64>,
    order: Vec<usize>,
    scratch_positions: Vec<Position>,
    scratch_velocities: Vec<Velocity>,
    scratch_masses: Vec<Mass>,
    scratch_inv_masses: Vec<Mass>,
    scratch_colors: Vec<Color>,
}

impl MortonSort {
    pub fn new() -> Self {
        Self {
            codes: Vec::new(),
            order: Vec::new(),
            scratch_positions: Vec::new(),
            scratch_velocities: Vec::new(),
            scratch_masses: Vec::new(),
            scratch_inv_masses: Vec::new(),
            scratch_colors: Vec::new(),
        }
    }

    pub fn sort(&mut self, particles: &mut Particles) {
        let n = particles.len();
        if n < 2 {
            return;
        }

        // Bounding box of the particles: the quantization domain for the Morton codes
        let (min_x, min_y, max_x, max_y) = particles
            .positions
            .par_iter()
            .fold(
                || {
                    (
                        Scalar::INFINITY,
                        Scalar::INFINITY,
                        Scalar::NEG_INFINITY,
                        Scalar::NEG_INFINITY,
                    )
                },
                |(min_x, min_y, max_x, max_y), p| {
                    (
                        min_x.min(p.x),
                        min_y.min(p.y),
                        max_x.max(p.x),
                        max_y.max(p.y),
                    )
                },
            )
            .reduce(
                || {
                    (
                        Scalar::INFINITY,
                        Scalar::INFINITY,
                        Scalar::NEG_INFINITY,
                        Scalar::NEG_INFINITY,
                    )
                },
                |a, b| (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3)),
            );

        let range_x = (max_x - min_x).max(Scalar::MIN_POSITIVE);
        let range_y = (max_y - min_y).max(Scalar::MIN_POSITIVE);
        let scale = (1u64 << 21) as Scalar;

        self.codes.clear();
        self.codes.resize(n, 0);
        self.codes.par_iter_mut().enumerate().for_each(|(i, code)| {
            let p = particles.positions[i];
            let ux = (((p.x - min_x) / range_x) * scale) as u64;
            let uy = (((p.y - min_y) / range_y) * scale) as u64;
            *code = morton_code(ux as u32, uy as u32);
        });

        self.order.clear();
        self.order.extend(0..n);
        // Tie-break on the index: the sort is unstable, equal Morton codes must
        // still produce a deterministic order
        self.order.par_sort_by_key(|&i| (self.codes[i], i));

        self.gather(particles);
    }

    fn gather(&mut self, particles: &mut Particles) {
        let n = particles.len();
        let order = &self.order;

        self.scratch_positions.clear();
        self.scratch_positions.resize(n, Position::zeros());
        self.scratch_positions
            .par_iter_mut()
            .enumerate()
            .for_each(|(j, dst)| *dst = particles.positions[order[j]]);

        self.scratch_velocities.clear();
        self.scratch_velocities.resize(n, Velocity::zeros());
        self.scratch_velocities
            .par_iter_mut()
            .enumerate()
            .for_each(|(j, dst)| *dst = particles.velocities[order[j]]);

        self.scratch_masses.clear();
        self.scratch_masses.resize(n, 0.0);
        self.scratch_masses
            .par_iter_mut()
            .enumerate()
            .for_each(|(j, dst)| *dst = particles.masses[order[j]]);

        self.scratch_inv_masses.clear();
        self.scratch_inv_masses.resize(n, 0.0);
        self.scratch_inv_masses
            .par_iter_mut()
            .enumerate()
            .for_each(|(j, dst)| *dst = particles.inv_masses[order[j]]);

        self.scratch_colors.clear();
        self.scratch_colors.resize(n, Color::BLACK);
        self.scratch_colors
            .par_iter_mut()
            .enumerate()
            .for_each(|(j, dst)| *dst = particles.colors[order[j]]);

        std::mem::swap(&mut particles.positions, &mut self.scratch_positions);
        std::mem::swap(&mut particles.velocities, &mut self.scratch_velocities);
        std::mem::swap(&mut particles.masses, &mut self.scratch_masses);
        std::mem::swap(&mut particles.inv_masses, &mut self.scratch_inv_masses);
        std::mem::swap(&mut particles.colors, &mut self.scratch_colors);
    }
}

impl System for MortonSort {
    fn update(&mut self, particles: &mut Particles, _dt: Time) {
        self.sort(particles);
    }
}
