use std::sync::{Arc, RwLock};

use nalgebra::Vector2;
use rayon::prelude::*;

use super::{
    areas::{Area, Rect},
    forces::{Drag, Force as ForceTrait, Gravity, Repulsion},
    particles::Particles,
    types::{Force, Mass, Position, Scalar, Velocity},
};

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
    fn barnes_hut(
        nodes: &[QuadTreeNode],
        stack: &mut Vec<usize>,
        gravity: &Gravity,
        repulsion: &Repulsion,
        drag: &Drag,
        theta: Scalar,
        particle: usize,
        particles: &Particles,
        force: &mut Force,
    ) {
        let _span = tracy_client::span!("Particle");
        let (mut leaf, mut approx, mut traverse) = (0, 0, 0);

        stack.clear();
        stack.push(0); // root

        let pos = particles.positions[particle];
        let vel = particles.velocities[particle];
        let mass = particles.masses[particle];

        while let Some(node_idx) = stack.pop() {
            let node = &nodes[node_idx];
            if node.childs.is_empty() {
                // Leaf node: Calculate the force directly between the particles if not the same particle
                leaf += 1;
                for (((&other, &other_pos), &other_vel), &other_mass) in node
                    .indexes
                    .iter()
                    .zip(&node.particles.positions)
                    .zip(&node.particles.velocities)
                    .zip(&node.particles.masses)
                {
                    if other == particle {
                        continue;
                    }

                    *force += gravity.calc_force(pos, other_pos, mass, other_mass);
                    *force += repulsion.calc_force(pos, other_pos);
                    *force += drag.calc_force(pos, other_pos, vel, other_vel);
                }
            } else if (node.scale / (node.center_of_mass - pos).norm()) < theta {
                // Barnes-Hut criterion satisfied: Approximate the force
                approx += 1;
                *force += gravity.calc_force(pos, node.center_of_mass, mass, node.total_mass);
                *force += repulsion.calc_force(pos, node.center_of_mass);
                *force += drag.calc_force(pos, node.center_of_mass, vel, node.average_velocity);
            } else {
                // Barnes-Hut criterion not satisfied: Traverse the children
                traverse += 1;
                for &child_idx in node.childs.iter() {
                    stack.push(child_idx);
                }
            }
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

        forces.par_iter_mut().enumerate().for_each_init(
            || Vec::with_capacity(max_depth_reached * 4 + 1),
            |stack, (i, force)| {
                Self::barnes_hut(
                    nodes,
                    stack,
                    &self.gravity,
                    &self.repulsion,
                    &self.drag,
                    self.theta,
                    i,
                    particles,
                    force,
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
