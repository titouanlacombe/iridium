use std::sync::{Arc, RwLock};

use nalgebra::Vector2;
use rayon::prelude::*;

use super::{
    areas::{Area, Rect},
    forces::{Drag, Force as ForceTrait, Gravity, Repulsion},
    particles::Particles,
    types::{
        mask_to_01, masked, repulsion_inv_pow, Force, Mass, Position, Scalar, Simd, SimdVec,
        Velocity,
    },
};

// Particles sharing one tree traversal. Morton-sorted particles make batches
// spatially coherent; the batch is one SIMD register (f64x4 for f64, f32x8 for
// f32) so pair forces vectorize across the lanes.
const BATCH: usize = <Simd as SimdVec>::LANES;

pub struct QuadTreeNode {
    pub rect: Rect,
    pub particles: Particles,
    pub indexes: Vec<usize>,
    // Indices into QuadTree::nodes (arena). Empty for leaves.
    pub childs: Vec<usize>,

    // For Barnes-Hut
    pub center_of_mass: Position,
    pub average_velocity: Velocity,
    pub total_mass: Mass,
    pub scale: Scalar,
}

impl Default for QuadTreeNode {
    fn default() -> Self {
        Self::new(Rect::new(Vector2::new(0.0, 0.0), Vector2::new(0.0, 0.0)))
    }
}

impl QuadTreeNode {
    pub fn new(rect: Rect) -> Self {
        let scale = rect.size.norm();
        Self {
            rect,
            particles: Particles::new_empty(),
            indexes: Vec::new(),
            childs: Vec::new(),
            center_of_mass: Vector2::new(0.0, 0.0),
            average_velocity: Vector2::new(0.0, 0.0),
            total_mass: 0.0,
            scale,
        }
    }
}

pub struct QuadTree {
    // Arena: all nodes live in one contiguous Vec, referenced by index.
    // nodes[0] is always the root.
    pub nodes: Vec<QuadTreeNode>,
    max_particles: usize,
    // TODO refactor forces (optional)
    gravity: Gravity,
    repulsion: Repulsion,
    drag: Drag,
    theta: Scalar, // Barnes-Hut (0.0: no approximation, 1.0: full approximation)

    // Max depth behavior
    max_depth: Option<usize>,
    max_depth_panics: bool,
}

impl QuadTree {
    pub fn new(
        rect: Rect,
        max_particles: usize,
        gravity: Gravity,
        repulsion: Repulsion,
        drag: Drag,
        theta: Scalar,
        max_depth: Option<usize>,
        max_depth_panics: bool,
    ) -> Self {
        Self {
            nodes: vec![QuadTreeNode::new(rect)],
            max_particles,
            gravity,
            repulsion,
            drag,
            theta,
            max_depth,
            max_depth_panics,
        }
    }

    pub fn root(&self) -> &QuadTreeNode {
        &self.nodes[0]
    }

    pub fn leaves(&self) -> impl Iterator<Item = &QuadTreeNode> {
        self.nodes.iter().filter(|node| node.childs.is_empty())
    }

    // Drop unreachable nodes (dead subtrees) and reindex the arena.
    // Runs once per insert so the arena stays bounded; its capacity is retained
    // so re-insertion does not reallocate.
    fn compact(&mut self) {
        let n = self.nodes.len();
        // remap[old] = new position after compaction; usize::MAX = unreachable,
        // which also serves as the visited marker for the DFS (and makes any
        // accidental read of a dead child panic loudly via out-of-bounds).
        let mut remap = vec![usize::MAX; n];
        let mut stack = vec![0usize];
        while let Some(idx) = stack.pop() {
            if remap[idx] != usize::MAX {
                continue;
            }
            remap[idx] = 0; // mark visited, actual position assigned below
            stack.extend(self.nodes[idx].childs.iter().copied());
        }

        // Assign final positions in index order (the compaction moves in index order)
        let mut new_len = 0;
        for i in 0..n {
            if remap[i] != usize::MAX {
                remap[i] = new_len;
                new_len += 1;
            }
        }

        // Reindex children before moving nodes around
        for i in 0..n {
            if remap[i] != usize::MAX {
                for child in &mut self.nodes[i].childs {
                    *child = remap[*child];
                }
            }
        }

        // Compact in place: position r still holds its original node at iteration r,
        // and slots < w have already been vacated (or were dead), so mem::take is safe.
        let mut w = 0;
        for r in 0..n {
            if remap[r] != usize::MAX {
                if w != r {
                    self.nodes[w] = std::mem::take(&mut self.nodes[r]);
                }
                w += 1;
            }
        }
        self.nodes.truncate(w);
    }

    // Returns (max_depth reached, number of nodes) computed during the traversal
    pub fn insert_particles(&mut self, particles: &Particles) -> (usize, usize) {
        let _span = tracy_client::span!("Insert Particles");

        let mut stack = Vec::new();
        stack.push((0, 0usize, (0..particles.len()).collect::<Vec<_>>()));

        let (mut max_depth_reached, mut nodes) = (0, 0);

        // TODO maybe parallelize
        while let Some((depth, node_idx, indexes)) = stack.pop() {
            max_depth_reached = max_depth_reached.max(depth);
            nodes += 1;
            let mut is_leaf = false;
            {
                let node = &mut self.nodes[node_idx];

                // Reset node
                node.center_of_mass = Vector2::new(0.0, 0.0);
                node.average_velocity = Vector2::new(0.0, 0.0);
                node.total_mass = 0.0;

                // Compute center of mass and total mass
                indexes.iter().for_each(|&particle_index| {
                    node.center_of_mass +=
                        particles.positions[particle_index] * particles.masses[particle_index];
                    node.average_velocity += particles.velocities[particle_index];
                    node.total_mass += particles.masses[particle_index];
                });
                node.center_of_mass /= node.total_mass;
                node.average_velocity /= indexes.len() as Scalar;

                // Check if we reached the maximum depth
                if let Some(max_depth) = self.max_depth {
                    if depth >= max_depth {
                        if self.max_depth_panics {
                            panic!("Max depth reached");
                        }
                        is_leaf = true;
                    }
                }

                if !is_leaf && indexes.len() <= self.max_particles {
                    is_leaf = true;
                }

                if is_leaf {
                    // Leaf node
                    // Copy particles (worth the spent time here when iterating in barnes hut)
                    node.particles.copy_from_indexes(&indexes, particles);
                    node.indexes = indexes;
                    node.childs.clear(); // dead children are freed by compaction
                    continue;
                }

                // Branch node
                node.particles.clear();
                node.indexes.clear();
            }

            // Ensure 4 children exist in the arena (reuse them from previous frames)
            if self.nodes[node_idx].childs.is_empty() {
                let (position, size) = {
                    let node = &self.nodes[node_idx];
                    (node.rect.position, node.rect.size)
                };
                let half_size = size / 2.0;
                let child_start = self.nodes.len();
                for i in 0..4 {
                    self.nodes.push(QuadTreeNode::new(Rect::new(
                        position
                            + Vector2::new(
                                (i % 2) as Scalar * half_size.x,
                                (i / 2) as Scalar * half_size.y,
                            ),
                        half_size,
                    )));
                }
                let node = &mut self.nodes[node_idx];
                node.childs.extend(child_start..child_start + 4);
            }

            // Particle redistribution
            // TODO parallelize
            let mut childs_indexes = vec![Vec::new(); 4];
            for particle_index in indexes.into_iter() {
                let mut child_num = 0;
                for (i, child_idx) in self.nodes[node_idx].childs.iter().skip(1).enumerate() {
                    if self.nodes[*child_idx].rect.contain(particles.positions[particle_index]) {
                        child_num = i + 1;
                        break;
                    }
                }
                childs_indexes[child_num].push(particle_index);
            }

            // Insert particles in children
            for (i, sub_indexes) in childs_indexes.into_iter().enumerate() {
                let child_idx = self.nodes[node_idx].childs[i];
                stack.push((depth + 1, child_idx, sub_indexes));
            }
        }

        self.compact();

        (max_depth_reached, nodes)
    }

    #[inline]
    fn barnes_hut_batch<V: SimdVec<Scalar = Scalar>>(
        nodes: &[QuadTreeNode],
        stack: &mut Vec<(usize, u8)>,
        gravity: &Gravity,
        repulsion: &Repulsion,
        drag: &Drag,
        theta: Scalar,
        start: usize,
        count: usize,
        particles: &Particles,
        forces: &mut [Force],
    ) {
        let _span = tracy_client::span!("Particle Batch");
        let (mut leaf, mut approx, mut traverse) = (0, 0, 0);

        // Packed batch state: one SIMD register per component.
        let mut px = [0.0; 8];
        let mut py = [0.0; 8];
        let mut vx = [0.0; 8];
        let mut vy = [0.0; 8];
        let mut mass = [0.0; 8];
        for k in 0..count {
            let i = start + k;
            px[k] = particles.positions[i].x;
            py[k] = particles.positions[i].y;
            vx[k] = particles.velocities[i].x;
            vy[k] = particles.velocities[i].y;
            mass[k] = particles.masses[i];
        }
        let (px, py, vx, vy, mass) = (
            V::from_8(&px),
            V::from_8(&py),
            V::from_8(&vx),
            V::from_8(&vy),
            V::from_8(&mass),
        );
        let mut fx = V::splat(0.0);
        let mut fy = V::splat(0.0);
        let one = V::splat(1.0);
        let zero = V::splat(0.0);

        stack.clear();
        // Root mask: first `count` bits set. u8::MAX >> (8 - count) instead of
        // (1 << count) - 1: for count == 8 the latter yields 0 (1u8 << 8 == 1).
        stack.push((0, u8::MAX >> (8 - count)));

        while let Some((node_idx, mask)) = stack.pop() {
            let node = &nodes[node_idx];

            // 1.0 for the lanes still traversing this subtree (from the stack entry mask)
            let mut subtree = [0.0; 8];
            for k in 0..count {
                if mask & (1 << k) != 0 {
                    subtree[k] = 1.0;
                }
            }
            let subtree = V::from_8(&subtree);

            if node.childs.is_empty() {
                // Leaf node: direct pairs between batch particles and leaf particles,
                // vectorized across the batch lanes
                leaf += 1;
                for (((&other, &other_pos), &other_vel), &other_mass) in node
                    .indexes
                    .iter()
                    .zip(&node.particles.positions)
                    .zip(&node.particles.velocities)
                    .zip(&node.particles.masses)
                {                    // Zero the lane of the batch's own particle (self-interaction)
                    let mut q_mask = subtree;
                    if other >= start && other < start + count {
                        let mut arr = [0.0; 8];
                        q_mask.write_8(&mut arr);
                        arr[other - start] = 0.0;
                        q_mask = V::from_8(&arr);
                    }

                    let dx = px - V::splat(other_pos.x);
                    let dy = py - V::splat(other_pos.y);
                    let r2 = dx * dx + dy * dy;
                    let r = r2.sqrt();

                    // Gravity. Validity masks select, not multiply: near-zero r
                    // gives inf/NaN, and inf * 0.0 is NaN.
                    let g_valid = mask_to_01(r.mask_ge(V::splat(gravity.epsilon)));
                    let r3 = r * r2;
                    let g_scale = -V::splat(gravity.coef) * mass * V::splat(other_mass) / r3;
                    fx += masked(q_mask, masked(g_valid, g_scale * dx));
                    fy += masked(q_mask, masked(g_valid, g_scale * dy));

                    // Repulsion: r^(-power) overflows to inf for tiny r in f32
                    let rep_valid = mask_to_01(r.mask_ge(V::splat(repulsion.epsilon)));
                    let rep_scale = V::splat(repulsion.coef) * repulsion_inv_pow(repulsion.power, r);
                    fx += masked(q_mask, masked(rep_valid, rep_scale * dx));
                    fy += masked(q_mask, masked(rep_valid, rep_scale * dy));

                    // Drag
                    let drag_valid = mask_to_01(r.mask_le(V::splat(drag.distance)))
                        * mask_to_01(r.mask_gt(zero));
                    // Mask the ratio first: distance == 0 would divide by zero
                    let dist_ratio = masked(drag_valid, r / V::splat(drag.distance));
                    let dist_coef = one - dist_ratio * dist_ratio;
                    let dvx = vx - V::splat(other_vel.x);
                    let dvy = vy - V::splat(other_vel.y);
                    let drag_scale = -V::splat(drag.coef) * dist_coef;
                    fx += masked(q_mask, masked(drag_valid, drag_scale * dvx));
                    fy += masked(q_mask, masked(drag_valid, drag_scale * dvy));
                }
            } else {
                // Barnes-Hut criterion, vectorized across the batch lanes.
                // dx = pos - com (matches calc_force's distance_v = pos1 - pos2)
                let dx = px - V::splat(node.center_of_mass.x);
                let dy = py - V::splat(node.center_of_mass.y);
                let dist2 = dx * dx + dy * dy;
                let dist = dist2.sqrt();
                let met = (V::splat(node.scale) / dist).mask_lt(V::splat(theta));
                let met_bits = met.to_bitmask();

                let mut child_mask = 0u8;
                for k in 0..count {
                    if mask & (1 << k) == 0 {
                        continue;
                    }
                    if met_bits & (1 << k) != 0 {
                        approx += 1;
                    } else {
                        child_mask |= 1 << k;
                    }
                }
                if child_mask != 0 {
                    traverse += 1;
                    for &child_idx in node.childs.iter() {
                        stack.push((child_idx, child_mask));
                    }
                }

                // Approximation contribution, masked to the satisfied lanes.
                // Validity masks select, not multiply: near-zero dist gives inf/NaN.
                let m = subtree * mask_to_01(met);

                let g_valid = mask_to_01(dist.mask_ge(V::splat(gravity.epsilon)));
                let r3 = dist * dist2;
                let g_scale = -V::splat(gravity.coef) * mass * V::splat(node.total_mass) / r3;
                fx += masked(m, masked(g_valid, g_scale * dx));
                fy += masked(m, masked(g_valid, g_scale * dy));

                let rep_valid = mask_to_01(dist.mask_ge(V::splat(repulsion.epsilon)));
                let rep_scale = V::splat(repulsion.coef) * repulsion_inv_pow(repulsion.power, dist);
                fx += masked(m, masked(rep_valid, rep_scale * dx));
                fy += masked(m, masked(rep_valid, rep_scale * dy));

                let drag_valid = mask_to_01(dist.mask_le(V::splat(drag.distance)))
                    * mask_to_01(dist.mask_gt(zero));
                // Mask the ratio first: distance == 0 would divide by zero
                let dist_ratio = masked(drag_valid, dist / V::splat(drag.distance));
                let dist_coef = one - dist_ratio * dist_ratio;
                let dvx = vx - V::splat(node.average_velocity.x);
                let dvy = vy - V::splat(node.average_velocity.y);
                let drag_scale = -V::splat(drag.coef) * dist_coef;
                fx += masked(m, masked(drag_valid, drag_scale * dvx));
                fy += masked(m, masked(drag_valid, drag_scale * dvy));
            }
        }

        // Write back. Accumulates: forces may already hold contributions from
        // other force types (Physics applies several forces to one buffer).
        let mut fx_arr = [0.0; 8];
        let mut fy_arr = [0.0; 8];
        fx.write_8(&mut fx_arr);
        fy.write_8(&mut fy_arr);
        for k in 0..count {
            forces[k] += Vector2::new(fx_arr[k], fy_arr[k]);
        }

        _span.emit_text(
            format!("Leaf: {}, Approx: {}, Traverse: {}", leaf, approx, traverse).as_str(),
        );
    }

    pub fn barnes_hut_particles(&mut self, particles: &Particles, forces: &mut Vec<Force>) {
        let _span = tracy_client::span!("Barnes-Hut");
        _span.emit_value(particles.len() as u64);

        // Make sure quadtree is up to date, and get the max depth for the stack hint
        let (max_depth_reached, _nodes) = self.insert_particles(particles);
        let nodes = &self.nodes;

        // One tree traversal per batch of BATCH particles. Morton-sorted particles
        // (see MortonSort) make the batches spatially coherent; the batch is one
        // SIMD register, so pair forces vectorize across the lanes.
        forces.par_chunks_mut(BATCH).enumerate().for_each_init(
            || Vec::with_capacity(max_depth_reached * 4 + 1),
            |stack, (chunk_idx, chunk)| {
                Self::barnes_hut_batch::<Simd>(
                    nodes,
                    stack,
                    &self.gravity,
                    &self.repulsion,
                    &self.drag,
                    self.theta,
                    chunk_idx * BATCH,
                    chunk.len(),
                    particles,
                    chunk,
                );
            },
        );
    }
}

pub struct QuadtreeForces {
    quadtree: Arc<RwLock<QuadTree>>,
}

impl QuadtreeForces {
    pub fn new(quadtree: Arc<RwLock<QuadTree>>) -> Self {
        Self { quadtree }
    }
}

impl ForceTrait for QuadtreeForces {
    fn apply(&mut self, particles: &Particles, forces: &mut Vec<Force>) {
        let mut quadtree = self.quadtree.write().unwrap();
        quadtree.barnes_hut_particles(particles, forces);
    }
}
