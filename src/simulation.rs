use crate::particle::Particle;

pub struct Simulation {
    particles: Vec<Particle>,
}

impl Simulation {
    pub fn new(particles: Vec<Particle>) -> Simulation {
        Simulation { particles }
    }

    pub fn update(&mut self) {
        for particle in &mut self.particles {
            particle.update();
        }
    }
}
