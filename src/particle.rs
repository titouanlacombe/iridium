use nalgebra::Vector2;

use crate::generators::Generator;

pub struct Particles {
    pub positions: Vec<Vector2<f32>>,
    pub velocities: Vec<Vector2<f32>>,
    pub masses: Vec<f32>,
}

impl Particles {
    pub fn new(
        positions: Vec<Vector2<f32>>,
        velocities: Vec<Vector2<f32>>,
        masses: Vec<f32>,
    ) -> Self {
        Self {
            positions,
            velocities,
            masses,
        }
    }

    pub fn new_empty() -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            masses: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Vector2<f32>, &Vector2<f32>, &f32)> {
        self.positions
            .iter()
            .zip(self.velocities.iter())
            .zip(self.masses.iter())
            .map(|((p, v), m)| (p, v, m))
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut Vector2<f32>, &mut Vector2<f32>, &mut f32)> {
        self.positions
            .iter_mut()
            .zip(self.velocities.iter_mut())
            .zip(self.masses.iter_mut())
            .map(|((p, v), m)| (p, v, m))
    }

    pub fn swap_remove(&mut self, i: usize) {
        self.positions.swap_remove(i);
        self.velocities.swap_remove(i);
        self.masses.swap_remove(i);
    }
}

pub trait ParticleFactory {
    fn create(&mut self, n: usize, particles: &mut Particles);
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
    fn create(&mut self, n: usize, particles: &mut Particles) {
        self.position_generator
            .generate(n, &mut particles.positions);
        self.velocity_generator
            .generate(n, &mut particles.velocities);
        self.mass_generator.generate(n, &mut particles.masses);
    }
}
