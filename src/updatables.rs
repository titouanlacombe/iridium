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

// Handle the case where the rate is not an integer
// Making the rate smooth & accurate across steps
fn smooth_rate(rate: f32, dt: f32) -> usize {
    let n = rate * dt;
    let mut quotient = n as usize;

    // Remainder
    if rand::random::<f32>() < n - quotient as f32 {
        quotient += 1;
    }

    quotient
}

impl Updatable for ConstantConsumer {
    fn update(&mut self, particles: &mut Vec<Particle>, dt: f32) {
        let quotient = smooth_rate(self.rate, dt);

        let mut to_remove = Vec::new();
        for (i, particle) in particles.iter_mut().enumerate() {
            if self.area.contains(particle.position) {
                to_remove.push(i);

                if to_remove.len() >= quotient {
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
        let quotient = smooth_rate(self.rate, dt);

        for _ in 0..quotient {
            particles.push(self.p_factory.create());
        }
    }
}
