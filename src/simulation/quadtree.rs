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
    pub particles: Vec<usize>,
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
            particles: Vec::new(),
            childs: Vec::new(),
            center_of_mass: Vector2::new(0.0, 0.0),
            average_velocity: Vector2::new(0.0, 0.0),
            total_mass: 0.0,
            scale,
        }
    }

    pub fn create_childs(&mut self) {
        let half_size = self.rect.size / 2.0;
        self.childs.reserve(4);
        for i in 0..4 {
            self.childs.push(QuadTreeNode::new(Rect::new(
                self.rect.position
                    + Vector2::new((i % 2) as f64 * half_size.x, (i / 2) as f64 * half_size.y),
                half_size,
            )));
        }
    }

    pub fn insert_particles(
        &mut self,
        mut indexes: Vec<usize>,
        positions: &Vec<Position>,
        velocities: &Vec<Velocity>,
        masses: &Vec<Mass>,
        max_particles: usize,
    ) {
        // Reset node
        self.center_of_mass = Vector2::new(0.0, 0.0);
        self.average_velocity = Vector2::new(0.0, 0.0);
        self.total_mass = 0.0;

        // Compute center of mass and total mass
        indexes.iter().for_each(|&particle_index| {
            self.center_of_mass += positions[particle_index] * masses[particle_index];
            self.average_velocity += velocities[particle_index];
            self.total_mass += masses[particle_index];
        });
        self.center_of_mass /= self.total_mass;
        self.average_velocity /= indexes.len() as f64;

        if indexes.len() <= max_particles {
            // Leaf node
            self.particles = indexes; // Take ownership of indexes
            self.childs.clear(); // Drop childs if necessary (pruning)
            self.childs.shrink_to_fit();
            return;
        }

        // Branch node
        self.particles.clear(); // Drop particles if necessary
        self.particles.shrink_to_fit();

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
                if child.rect.contain(positions[particle_index]) {
                    child_num = i + 1;
                    break;
                }
            }
            childs_indexes[child_num].push(particle_index);
        }

        // Insert particles in childs
        // TODO maybe parallelize
        for (child, indexes) in self.childs.iter_mut().zip(childs_indexes) {
            child.insert_particles(indexes, positions, velocities, masses, max_particles);
        }
    }
}

pub struct QuadTree {
    pub root: QuadTreeNode,
    // allocator: Arena<QuadTreeNode>,
    max_particles: usize,
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

    pub fn insert_particles(&mut self, particles: &Particles) {
        // Insert particles (will prune the tree if necessary)
        self.root.insert_particles(
            (0..particles.len()).collect::<Vec<_>>(),
            &particles.positions,
            &particles.velocities,
            &particles.masses,
            self.max_particles,
        );
    }

    #[inline]
    fn barnes_hut(
        root: &QuadTreeNode,
        gravity: &Gravity,
        repulsion: &Repulsion,
        drag: &Drag,
        theta: f64,
        particle: usize,
        positions: &Vec<Position>,
        velocities: &Vec<Velocity>,
        masses: &Vec<Mass>,
        force: &mut Force,
    ) {
        let mut stack = Vec::new();
        stack.push(root);

        let pos = positions[particle];
        let vel = velocities[particle];
        let mass = masses[particle];

        while let Some(node) = stack.pop() {
            if node.childs.is_empty() {
                // Leaf node: Calculate the force directly between the particles if not the same particle
                for &other in &node.particles {
                    if other == particle {
                        continue;
                    }

                    *force += gravity.calc_force(pos, positions[other], mass, masses[other]);
                    *force += repulsion.calc_force(pos, positions[other]);
                    *force += drag.calc_force(pos, positions[other], vel, velocities[other]);
                }
            } else if (node.scale / (node.center_of_mass - pos).norm()) < theta {
                // Barnes-Hut criterion satisfied: Approximate the force
                *force += gravity.calc_force(pos, node.center_of_mass, mass, node.total_mass);
                *force += repulsion.calc_force(pos, node.center_of_mass);
                *force += drag.calc_force(pos, node.center_of_mass, vel, node.average_velocity);
            } else {
                // Barnes-Hut criterion not satisfied: Traverse the children
                for child in node.childs.iter() {
                    stack.push(child);
                }
            }
        }
    }

    pub fn barnes_hut_particles(&self, particles: &Particles, forces: &mut Vec<Force>) {
        forces.par_iter_mut().enumerate().for_each(|(i, force)| {
            Self::barnes_hut(
                &self.root,
                &self.gravity,
                &self.repulsion,
                &self.drag,
                self.theta,
                i,
                &particles.positions,
                &particles.velocities,
                &particles.masses,
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
        quadtree.insert_particles(particles);
        quadtree.barnes_hut_particles(particles, forces);
    }
}
