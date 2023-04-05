use crate::{
    areas::Area,
    particle::{Particle, ParticleFactory},
};

pub trait System {
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

impl System for ConstantConsumer {
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

impl System for ConstantEmitter {
    fn update(&mut self, particles: &mut Vec<Particle>, dt: f32) {
        let quotient = smooth_rate(self.rate, dt);

        for _ in 0..quotient {
            particles.push(self.p_factory.create());
        }
    }
}

pub struct Wall {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
    pub restitution: f32,
}

impl System for Wall {
    fn update(&mut self, particles: &mut Vec<Particle>, _dt: f32) {
        for particle in particles.iter_mut() {
            let p_pos = &mut particle.position;
            let p_vel = &mut particle.velocity;

            if p_pos.x < self.x_min {
                p_pos.x = self.x_min;
                p_vel.x = -p_vel.x * self.restitution;
            } else if p_pos.x > self.x_max {
                p_pos.x = self.x_max;
                p_vel.x = -p_vel.x * self.restitution;
            }

            if p_pos.y < self.y_min {
                p_pos.y = self.y_min;
                p_vel.y = -p_vel.y * self.restitution;
            } else if p_pos.y > self.y_max {
                p_pos.y = self.y_max;
                p_vel.y = -p_vel.y * self.restitution;
            }
        }
    }
}

pub struct Loop {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

impl System for Loop {
    fn update(&mut self, particles: &mut Vec<Particle>, _dt: f32) {
        for particle in particles.iter_mut() {
            let p_pos = &mut particle.position;

            if p_pos.x < self.x_min {
                p_pos.x = self.x_max;
            } else if p_pos.x > self.x_max {
                p_pos.x = self.x_min;
            }

            if p_pos.y < self.y_min {
                p_pos.y = self.y_max;
            } else if p_pos.y > self.y_max {
                p_pos.y = self.y_min;
            }
        }
    }
}

pub struct Void {
    pub area: Box<dyn Area>,
}

impl System for Void {
    fn update(&mut self, particles: &mut Vec<Particle>, _dt: f32) {
        let mut to_remove = Vec::new();
        for (i, particle) in particles.iter_mut().enumerate() {
            if !self.area.contains(particle.position) {
                to_remove.push(i);
            }
        }

        for i in to_remove {
            particles.swap_remove(i);
        }
    }
}
