use std::sync::{Arc, RwLock};

use nalgebra::Vector2;
use rayon::prelude::*;

use super::{
    areas::{Area, Rect},
    forces::{Drag, Force as ForceTrait, Gravity, Repulsion},
    particles::Particles,
    types::{Force, Mass, Position, Velocity},
};

pub struct QuadTreeNode {
    pub rect: Rect,
    pub particles: Particles,
    pub indexes: Vec<usize>,
    pub childs: Vec<QuadTreeNode>,

    // For Barnes-Hut
    pub center_of_mass: Position,
    pub average_velocity: Velocity,
    pub total_mass: Mass,
    pub scale: f64,
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

    pub fn create_childs(&mut self) {
        let half_size = self.rect.size / 2.0;
        self.childs.reserve_exact(4);
        for i in 0..4 {
            self.childs.push(QuadTreeNode::new(Rect::new(
                self.rect.position
                    + Vector2::new((i % 2) as f64 * half_size.x, (i / 2) as f64 * half_size.y),
                half_size,
            )));
        }
    }

    fn _insert_particles<'a>(
        &'a mut self,
        stack: &mut Vec<(&'a mut Self, Vec<usize>)>,
        mut indexes: Vec<usize>,
        particles: &Particles,
        max_particles: usize,
    ) {
        // Reset node
        self.center_of_mass = Vector2::new(0.0, 0.0);
        self.average_velocity = Vector2::new(0.0, 0.0);
        self.total_mass = 0.0;

        // Compute center of mass and total mass
        indexes.iter().for_each(|&particle_index| {
            self.center_of_mass +=
                particles.positions[particle_index] * particles.masses[particle_index];
            self.average_velocity += particles.velocities[particle_index];
            self.total_mass += particles.masses[particle_index];
        });
        self.center_of_mass /= self.total_mass;
        self.average_velocity /= indexes.len() as f64;

        if indexes.len() <= max_particles {
            // Leaf node
            // Copy particles (worth the spent time here when iterating in barnes hut)
            self.particles.copy_from_indexes(&indexes, particles);
            self.indexes = indexes; // Take ownership of indexes
            self.indexes.shrink_to_fit();
            self.childs.clear(); // Drop childs if necessary (pruning)
            self.childs.shrink_to_fit();
            return;
        }

        // Branch node
        self.particles.clear(); // Drop particles if necessary
        self.particles.shrink_to_fit();
        self.indexes.clear(); // Drop indexes if necessary
        self.indexes.shrink_to_fit();

        // Create childs if necessary
        if self.childs.is_empty() {
            self.create_childs();
        }

        // Particle redistribution
        // TODO parallelize
        let mut childs_indexes = vec![Vec::new(); 4];
        for particle_index in indexes.drain(..) {
            let mut child_num = 0;
            for (i, child) in self.childs.iter().skip(1).enumerate() {
                if child.rect.contain(particles.positions[particle_index]) {
                    child_num = i + 1;
                    break;
                }
            }
            childs_indexes[child_num].push(particle_index);
        }

        // Insert particles in childs
        for (child, indexes) in self.childs.iter_mut().zip(childs_indexes) {
            stack.push((child, indexes));
        }
    }

    pub fn insert_particles<'a>(
        &'a mut self,
        indexes: Vec<usize>,
        particles: &Particles,
        max_particles: usize,
    ) {
        let _span = tracy_client::span!("Insert Particles");

        let mut stack = Vec::new();
        stack.push((self, indexes));

        // TODO maybe parallelize
        while let Some((node, indexes)) = stack.pop() {
            node._insert_particles(&mut stack, indexes, &particles, max_particles);
        }
    }
}

pub struct QuadTree {
    pub root: QuadTreeNode,
    // allocator: Arena<QuadTreeNode>,
    max_particles: usize,
    // TODO refactor forces (optional)
    gravity: Gravity,
    repulsion: Repulsion,
    drag: Drag,
    theta: f64, // Barnes-Hut (0.0: no approximation, 1.0: full approximation)
}

impl QuadTree {
    pub fn new(
        rect: Rect,
        max_particles: usize,
        gravity: Gravity,
        repulsion: Repulsion,
        drag: Drag,
        theta: f64,
    ) -> Self {
        Self {
            root: QuadTreeNode::new(rect),
            // allocator: Arena::new(),
            max_particles,
            gravity,
            repulsion,
            drag,
            theta,
        }
    }

    fn insert_particles(&mut self, particles: &Particles) {
        // Insert particles (will prune the tree if necessary)
        self.root.insert_particles(
            (0..particles.len()).collect::<Vec<_>>(),
            &particles,
            self.max_particles,
        );
    }

    // Traverse the tree and return max_depth and total number of nodes
    fn get_infos(&self) -> (usize, usize) {
        let mut stack = Vec::new();
        stack.push(&self.root);

        let mut max_depth = 0;
        let mut nodes = 0;

        while let Some(node) = stack.pop() {
            max_depth = max_depth.max(node.childs.len());
            nodes += 1;
            stack.reserve(node.childs.len());
            for child in node.childs.iter() {
                stack.push(child);
            }
        }

        (max_depth, nodes)
    }

    #[inline]
    fn barnes_hut(
        root: &QuadTreeNode,
        gravity: &Gravity,
        repulsion: &Repulsion,
        drag: &Drag,
        theta: f64,
        max_depth: usize,
        particle: usize,
        particles: &Particles,
        force: &mut Force,
    ) {
        let _span = tracy_client::span!("Particle");
        let (mut leaf, mut approx, mut traverse) = (0, 0, 0);

        // We know the maximum number of nodes we will traverse, so we can preallocate the stack
        let mut stack = Vec::with_capacity(max_depth * 3 + 1);
        stack.push(root);

        let pos = particles.positions[particle];
        let vel = particles.velocities[particle];
        let mass = particles.masses[particle];

        while let Some(node) = stack.pop() {
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
                for child in node.childs.iter() {
                    stack.push(child);
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

        // Make sure quadtree is up to date
        self.insert_particles(particles);

        // Compute max depth and total number of nodes
        let (max_depth, _nodes) = self.get_infos();

        forces.par_iter_mut().enumerate().for_each(|(i, force)| {
            Self::barnes_hut(
                &self.root,
                &self.gravity,
                &self.repulsion,
                &self.drag,
                self.theta,
                max_depth,
                i,
                particles,
                force,
            );
        });
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
