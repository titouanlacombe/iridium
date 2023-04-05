use crate::{
    areas::Area,
    particle::{Particle, ParticleFactory},
};

pub trait Updatable {
    fn update(&mut self, particles: &mut Vec<Particle>, dt: f32);
}

pub struct ConstantConsumer {
    pub area: Box<dyn Area>,
    pub rate: f32,
}

impl ConstantConsumer {
    pub fn new(area: Box<dyn Area>, rate: f32) -> Self {
        Self { area, rate }
    }
}

impl Updatable for ConstantConsumer {
    fn update(&mut self, particles: &mut Vec<Particle>, dt: f32) {
        let mut to_remove = Vec::new();
        let limit = (self.rate * dt) as usize;

        for (i, particle) in particles.iter_mut().enumerate() {
            if self.area.contains(particle.position) {
                to_remove.push(i);

                if to_remove.len() >= limit {
                    break;
                }
            }
        }

        for i in to_remove {
            particles.swap_remove(i);
        }
    }
}

pub struct ConstantEmitter {
    pub p_factory: Box<dyn ParticleFactory>,
    pub rate: f32,
}

impl ConstantEmitter {
    pub fn new(p_factory: Box<dyn ParticleFactory>, rate: f32) -> Self {
        Self { p_factory, rate }
    }
}

impl Updatable for ConstantEmitter {
    fn update(&mut self, particles: &mut Vec<Particle>, dt: f32) {
        let n = self.rate * dt;

        let quotient = n as usize;
        for _ in 0..quotient {
            particles.push(self.p_factory.create());
        }

        let remainder = n - quotient as f32;
        if rand::random::<f32>() < remainder {
            particles.push(self.p_factory.create());
        }
    }
}
