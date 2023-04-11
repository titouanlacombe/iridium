use nalgebra::Vector2;

use crate::generators::Generator;

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

pub trait ParticleFactory {
    fn create(&self, n: usize) -> Vec<Particle>;
}

pub struct GeneratorFactory {
    pub position_generator: Box<dyn Generator<Vector2<f32>>>,
    pub velocity_generator: Box<dyn Generator<Vector2<f32>>>,
    pub mass_generator: Box<dyn Generator<f32>>,
}

impl GeneratorFactory {
    pub fn new(
        position_generator: Box<dyn Generator<Vector2<f32>>>,
        velocity_generator: Box<dyn Generator<Vector2<f32>>>,
        mass_generator: Box<dyn Generator<f32>>,
    ) -> Self {
        Self {
            position_generator,
            velocity_generator,
            mass_generator,
        }
    }
}

impl ParticleFactory for GeneratorFactory {
    fn create(&self, n: usize) -> Vec<Particle> {
        let positions = self.position_generator.generate(n);
        let velocities = self.velocity_generator.generate(n);
        let masses = self.mass_generator.generate(n);

        positions
            .into_iter()
            .zip(velocities.into_iter())
            .zip(masses.into_iter())
            .map(|((position, velocity), mass)| Particle::new(position, velocity, mass))
            .collect()
    }
}
