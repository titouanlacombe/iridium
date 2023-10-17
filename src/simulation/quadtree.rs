use nalgebra::Vector2;
use rayon::prelude::*;

use super::{
    areas::{Area, Rect},
    forces::Gravity,
    particles::Particles,
    types::{Force, Mass, Position, Scalar},
};

pub struct QuadTreeNode {
    pub rect: Rect,
    pub childs: Vec<QuadTreeNode>,
    pub particles: Vec<usize>,
    pub nb_particles: usize, // Take childs into account

    // For Barnes-Hut
    pub total_mass: Mass,
    pub position_of_mass: Vector2<Scalar>,

    // Caches
    pub scale: f64,
    pub center_of_mass: Position,
}

// TODO create iterator for quadtreenode

impl QuadTreeNode {
    // TODO use allocator adapted to the problem
    pub fn new(rect: Rect, max_particles: usize) -> Self {
        let scale = rect.size.norm();

        Self {
            rect,
            childs: Vec::with_capacity(4),
            particles: Vec::with_capacity(max_particles / 4),
            nb_particles: 0,
            total_mass: 0.0,
            position_of_mass: Vector2::new(0.0, 0.0),
            scale,
            center_of_mass: Vector2::new(0.0, 0.0),
        }
    }

    fn should_divide(&self, max_particles: usize) -> bool {
        self.nb_particles > max_particles
    }

    fn find_target(&mut self, position: Position) -> &mut QuadTreeNode {
        let mut i = 0;
        while i < 4 {
            if self.childs[i].rect.contain(position) {
                break;
            }
            i += 1;
        }

        &mut self.childs[i]
    }

    fn subdivide(&mut self, particles: &Particles, max_particles: usize) {
        let half_size = self.rect.size / 2.0;

        for i in 0..4 {
            self.childs.push(QuadTreeNode::new(
                Rect::new(
                    self.rect.position
                        + Vector2::new((i % 2) as f64 * half_size.x, (i / 2) as f64 * half_size.y),
                    half_size,
                ),
                max_particles,
            ));
        }

        for particle in self.particles.iter() {
            // TODO fix borrow error
            self.find_target(particles.positions[*particle])
                .insert_particle(*particle, particles, max_particles);
        }

        // Free memory
        self.particles.shrink_to_fit();
    }

    fn merge(&mut self) {
        let mut stack = Vec::new();
        stack.extend(self.childs.iter_mut());

        while let Some(node) = stack.pop() {
            self.particles.extend(node.particles.drain(..));
            stack.extend(node.childs.iter_mut());
        }

        self.childs.clear();
    }

    pub fn prune(&mut self, max_particles: usize) {
        if !self.should_divide(max_particles) {
            self.merge();
        }

        for child in self.childs.iter_mut() {
            child.prune(max_particles);
        }
    }

    #[inline]
    pub fn insert_particle(&mut self, index: usize, particles: &Particles, max_particles: usize) {
        let position = particles.positions[index];
        let mass = particles.masses[index];

        // Update node data
        self.nb_particles += 1;
        self.position_of_mass += position * mass;
        self.total_mass += mass;

        if self.childs.is_empty() {
            // If leaf node, insert & check for subdivision
            self.particles.push(index);
            if self.should_divide(max_particles) {
                self.subdivide(particles, max_particles);
            }
            return;
        }

        // Insert particle in the correct child
        self.find_target(position)
            .insert_particle(index, particles, max_particles);
    }
}

pub struct QuadTree {
    root: QuadTreeNode,
    max_particles: usize,
    gravity: Gravity,
    theta: f64, // Barnes-Hut (0.0: no approximation, 1.0: full approximation)
}

impl QuadTree {
    pub fn new(rect: Rect, max_particles: usize, gravity: Gravity, theta: f64) -> Self {
        Self {
            root: QuadTreeNode::new(rect, max_particles),
            max_particles,
            gravity,
            theta,
        }
    }

    pub fn clear(&mut self) {
        let mut stack = Vec::new();
        stack.push(&mut self.root);

        while let Some(node) = stack.pop() {
            node.particles.clear();
            node.nb_particles = 0;
            node.position_of_mass = Position::new(0.0, 0.0);
            node.total_mass = 0.0;

            stack.extend(node.childs.iter_mut());
        }
    }

    pub fn prepare(&mut self) {
        let mut stack = Vec::new();
        stack.push(&mut self.root);

        while let Some(node) = stack.pop() {
            node.center_of_mass = node.position_of_mass / node.total_mass;

            stack.extend(node.childs.iter_mut());
        }
    }

    #[inline]
    fn barnes_hut(&self, index: usize, particles: &Particles, force: &mut Force) {
        let mut stack = Vec::new();
        stack.push(&self.root);

        let pos = particles.positions[index];
        let mass = particles.masses[index];

        while let Some(node) = stack.pop() {
            let distance = (node.center_of_mass - pos).norm();

            if node.childs.is_empty() {
                // Leaf node: Calculate the force directly between the particles if not the same particle
                for particle in &node.particles {
                    if *particle == index {
                        continue;
                    }

                    *force += self.gravity.newton(
                        pos,
                        particles.positions[*particle],
                        mass,
                        particles.masses[*particle],
                    );
                }
            } else if (node.scale / distance) < self.theta {
                // Barnes-Hut criterion satisfied: Approximate the force
                *force += self
                    .gravity
                    .newton(pos, node.center_of_mass, mass, node.total_mass);
            } else {
                // Barnes-Hut criterion not satisfied: Traverse the children
                for child in node.childs.iter() {
                    stack.push(child);
                }
            }
        }
    }

    pub fn gravity(&mut self, particles: &Particles, forces: &mut Vec<Force>) {
        // Clear tree (reset nodes but keep structure)
        self.clear();

        // Insert particles
        // TODO par iter
        (0..particles.len()).into_iter().for_each(|index| {
            self.root
                .insert_particle(index, particles, self.max_particles);
        });

        // Prune tree structure
        self.root.prune(self.max_particles);

        // Prepare tree for force calculation
        self.prepare();

        // Calculate forces
        forces.par_iter_mut().enumerate().for_each(|(i, force)| {
            self.barnes_hut(i, particles, force);
        });
    }
}
