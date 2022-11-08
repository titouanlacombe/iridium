use crate::particle::{Drain, Particle, Tap};

pub struct Simulation {
    pub particles: Vec<Particle>,
    pub taps: Vec<Tap>,
    pub drains: Vec<Drain>,
}

impl Simulation {
    pub fn new(particles: Vec<Particle>, taps: Vec<Tap>, drains: Vec<Drain>) -> Simulation {
        Simulation {
            particles,
            taps,
            drains,
        }
    }

    pub fn new_empty() -> Simulation {
        Simulation::new(Vec::new(), Vec::new(), Vec::new())
    }

    pub fn update(&mut self) {
        for particle in &mut self.particles {
            particle.update();
        }

        for tap in &mut self.taps {
            tap.update();
        }

        for drain in &mut self.drains {
            drain.update();
        }
    }

    pub fn add_particle(&mut self, particle: Particle) {
        self.particles.push(particle);
    }
}
