use nalgebra::Vector2;

use super::{
    areas::{Area, Rect},
    forces::Gravity,
    particles::Particles,
    types::{Force, Mass, Position, Scalar},
};

pub struct QuadTreeNode {
    pub rect: Rect,

    pub childs: Option<Box<[QuadTreeNode; 4]>>,
    pub particles: Vec<usize>,

    // For Barnes-Hut
    pub position_of_mass: Vector2<Scalar>,
    pub total_mass: Mass,
    pub center_of_mass: Option<Position>, // Cache
}

// TODO iterator
impl QuadTreeNode {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            childs: None,
            particles: Vec::new(),
            position_of_mass: Vector2::new(0.0, 0.0),
            total_mass: 0.0,
            center_of_mass: None,
        }
    }

    fn should_divide(&self, max_particles: usize) -> bool {
        self.particles.len() > max_particles
    }

    fn subdivide(&mut self, particles: &Particles, max_particles: usize) {
        let half_size = self.rect.size / 2.0;

        self.childs = Some(Box::new([
            QuadTreeNode::new(Rect::new(self.rect.position, half_size)),
            QuadTreeNode::new(Rect::new(
                self.rect.position + Vector2::new(half_size.x, 0.0),
                half_size,
            )),
            QuadTreeNode::new(Rect::new(
                self.rect.position + Vector2::new(0.0, half_size.y),
                half_size,
            )),
            QuadTreeNode::new(Rect::new(self.rect.position + half_size, half_size)),
        ]));

        for particle in self.particles.drain(..) {
            let mut inserted = false;
            for child in self.childs.as_mut().unwrap().iter_mut() {
                if child.rect.contain(particles.positions[particle]) {
                    child.insert_particle(particle, particles, max_particles);
                    inserted = true;
                    break;
                }
            }

            // If particle is not inserted, insert it in the first child
            if !inserted {
                // TODO check
                // println!("Not inserted");
                self.childs.as_mut().unwrap()[0].insert_particle(
                    particle,
                    particles,
                    max_particles,
                );
            }
        }
    }

    fn merge(&mut self) {
        let mut stack = Vec::new();
        stack.push(self);

        while let Some(node) = stack.pop() {
            self.particles.extend(node.particles.drain(..));

            if let Some(childs) = &mut node.childs {
                stack.extend(childs.iter_mut());
            }
        }

        self.childs = None;
    }

    pub fn prune(&mut self, max_particles: usize) {
        if !self.should_divide(max_particles) {
            self.merge();
        }

        if let Some(childs) = &mut self.childs {
            for child in childs.iter_mut() {
                child.prune(max_particles);
            }
        }
    }

    pub fn insert_particle(&mut self, index: usize, particles: &Particles, max_particles: usize) {
        let position = particles.positions[index];
        let mass = particles.masses[index];

        // Update position of mass and total mass
        self.position_of_mass += position * mass;
        self.total_mass += mass;

        if let Some(childs) = &mut self.childs {
            for child in childs.iter_mut() {
                if child.rect.contain(position) {
                    child.insert_particle(index, particles, max_particles);
                    return;
                }
            }
        } else {
            self.particles.push(index);
            if self.should_divide(max_particles) {
                self.subdivide(particles, max_particles);
            }
        }
    }

    fn get_center_of_mass(&mut self) -> Position {
        if let Some(center_of_mass) = self.center_of_mass {
            center_of_mass
        } else {
            let center_of_mass = self.position_of_mass / self.total_mass;
            self.center_of_mass = Some(center_of_mass);
            center_of_mass
        }
    }
}

pub struct QuadTree {
    root: QuadTreeNode,
    max_particles: usize,
    gravity: Gravity,
    theta: f64, // Barnes-Hut
}

impl QuadTree {
    pub fn new(rect: Rect, max_particles: usize, gravity: Gravity, theta: f64) -> Self {
        Self {
            root: QuadTreeNode::new(rect),
            max_particles,
            gravity,
            theta,
        }
    }

    pub fn reset(&mut self) {
        let mut stack = Vec::new();
        stack.push(&mut self.root);

        while let Some(node) = stack.pop() {
            node.particles.clear();
            node.position_of_mass = Position::new(0.0, 0.0);
            node.total_mass = 0.0;

            if let Some(childs) = &mut node.childs {
                stack.extend(childs.iter_mut());
            }
        }
    }

    pub fn insert_particles(&mut self, particles: &Particles) {
        for index in 0..particles.len() {
            self.root
                .insert_particle(index, particles, self.max_particles);
        }
    }

    fn barnes_hut(&self, index: usize, particles: &Particles, force: &mut Force) {
        let mut stack = Vec::new();
        stack.push(&mut self.root);

        let pos = particles.positions[index];
        let mass = particles.masses[index];

        while let Some(node) = stack.pop() {
            let center_of_mass = node.get_center_of_mass();
            let distance = (center_of_mass - pos).norm();
            let theta = self.theta * node.rect.size.norm() / distance;

            if theta < 1.0 {
                *force += self
                    .gravity
                    .newton(pos, center_of_mass, mass, node.total_mass);
            } else {
                if let Some(childs) = &node.childs {
                    for child in childs.iter_mut() {
                        if child.rect.contain(pos) {
                            stack.push(child);
                        }
                    }
                }
            }
        }
    }

    pub fn gravity(&self, particles: &Particles, forces: &mut Vec<Force>) {
        for index in 0..particles.len() {
            self.barnes_hut(index, particles, &mut forces[index]);
        }
    }
}
