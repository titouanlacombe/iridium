use crate::{
    areas::Area,
    particle::{ParticleFactory, Particles},
};

pub trait System {
    fn update(&mut self, particles: &mut Particles, dt: f32);
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
    fn update(&mut self, particles: &mut Particles, dt: f32) {
        let quotient = smooth_rate(self.rate, dt);

        let mut to_remove = Vec::new();
        for (i, position) in particles.positions.iter().enumerate() {
            if self.area.contains(*position) {
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
    fn update(&mut self, particles: &mut Particles, dt: f32) {
        let quotient = smooth_rate(self.rate, dt);
        self.p_factory.create(quotient, particles);
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
    fn update(&mut self, particles: &mut Particles, _dt: f32) {
        for (position, velocity, _mass) in particles.iter_mut() {
            if position.x < self.x_min {
                position.x = self.x_min;
                velocity.x = -velocity.x * self.restitution;
            } else if position.x > self.x_max {
                position.x = self.x_max;
                velocity.x = -velocity.x * self.restitution;
            }

            if position.y < self.y_min {
                position.y = self.y_min;
                velocity.y = -velocity.y * self.restitution;
            } else if position.y > self.y_max {
                position.y = self.y_max;
                velocity.y = -velocity.y * self.restitution;
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
    fn update(&mut self, particles: &mut Particles, _dt: f32) {
        for position in particles.positions.iter_mut() {
            if position.x < self.x_min {
                position.x = self.x_max;
            } else if position.x > self.x_max {
                position.x = self.x_min;
            }

            if position.y < self.y_min {
                position.y = self.y_max;
            } else if position.y > self.y_max {
                position.y = self.y_min;
            }
        }
    }
}

pub struct Void {
    pub area: Box<dyn Area>,
}

impl System for Void {
    fn update(&mut self, particles: &mut Particles, _dt: f32) {
        let mut to_remove = Vec::new();
        for (i, position) in particles.positions.iter().enumerate() {
            if !self.area.contains(*position) {
                to_remove.push(i);
            }
        }

        for i in to_remove {
            particles.swap_remove(i);
        }
    }
}
