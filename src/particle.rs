use nalgebra::Vector2;

use crate::areas::Area;

pub struct Particle {
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub mass: f32,
}

impl Particle {
    pub fn new(position: Vector2<f32>, velocity: Vector2<f32>, mass: f32) -> Self {
        Self {
            position,
            velocity,
            mass,
        }
    }
}

fn rand_range(min: f32, max: f32) -> f32 {
    min + (max - min) * rand::random::<f32>()
}

pub trait ParticleFactory {
    fn create(&self) -> Particle;
}

pub struct RandomFactory {
    // TODO use generators for rand ranges (constant generators, uniform generators, etc.)
    pub area: Box<dyn Area>,
    pub velocity_min: f32,
    pub velocity_max: f32,
    pub velocity_angle_min: f32,
    pub velocity_angle_max: f32,

    pub mass_min: f32,
    pub mass_max: f32,
}

// TODO Switch from box dyn to generic
impl RandomFactory {
    pub fn new(
        area: Box<dyn Area>,
        velocity_min: f32,
        velocity_max: f32,
        velocity_angle_min: f32,
        velocity_angle_max: f32,
        mass_min: f32,
        mass_max: f32,
    ) -> Self {
        Self {
            area,
            velocity_min,
            velocity_max,
            velocity_angle_min,
            velocity_angle_max,
            mass_min,
            mass_max,
        }
    }
}

impl ParticleFactory for RandomFactory {
    fn create(&self) -> Particle {
        let position = self.area.rand();

        let velocity_magn = rand_range(self.velocity_min, self.velocity_max);
        let velocity_angle = rand_range(self.velocity_angle_min, self.velocity_angle_max);
        let velocity = Vector2::new(
            velocity_magn * velocity_angle.cos(),
            velocity_magn * velocity_angle.sin(),
        );

        let mass = rand_range(self.mass_min, self.mass_max);

        Particle::new(position, velocity, mass)
    }
}
