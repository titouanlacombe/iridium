use nalgebra::Vector2;

use super::{
    areas::{Area, Rect},
    particles::Particles,
    types::{Mass, Position},
};

pub struct QuadTreeNode {
    pub rect: Rect,

    pub childs: Option<Box<[QuadTreeNode; 4]>>,
    pub particles: Vec<usize>,

    // For Barnes-Hut
    pub position_of_mass: Position,
    pub total_mass: Mass,
}

impl QuadTreeNode {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            childs: None,
            particles: Vec::new(),
            position_of_mass: Position::new(0.0, 0.0),
            total_mass: 0.0,
        }
    }

    pub fn should_divide(&self, max_particles: usize) -> bool {
        self.particles.len() > max_particles
    }

    pub fn reset(&mut self) {
        if let Some(childs) = &mut self.childs {
            for child in childs.iter_mut() {
                child.reset();
            }
        }

        self.particles.clear();
        self.position_of_mass = Position::new(0.0, 0.0);
        self.total_mass = 0.0;
    }

    pub fn subdivide(&mut self, particles: &Particles, max_particles: usize) {
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

    pub fn merge(&mut self) {
        if let Some(childs) = &mut self.childs {
            for child in childs.iter_mut() {
                child.merge();
                self.particles.append(&mut child.particles);
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
}

pub struct QuadTree {
    root: QuadTreeNode,
    max_particles: usize,
}

impl QuadTree {
    pub fn insert_particles(&mut self, particles: &Particles) {
        for index in 0..particles.len() {
            self.root
                .insert_particle(index, particles, self.max_particles);
        }
    }

    pub fn new(rect: Rect, max_particles: usize) -> Self {
        Self {
            root: QuadTreeNode::new(rect),
            max_particles,
        }
    }
}
