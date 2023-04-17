use crate::{
    generators::Generator,
    types::{Mass, Position, Velocity},
};

pub struct Particles {
    pub positions: Vec<Position>,
    pub velocities: Vec<Velocity>,
    pub masses: Vec<Mass>,
}

impl Particles {
    pub fn new(positions: Vec<Position>, velocities: Vec<Velocity>, masses: Vec<Mass>) -> Self {
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

    pub fn iter(&self) -> impl Iterator<Item = (&Position, &Velocity, &Mass)> {
        self.positions
            .iter()
            .zip(self.velocities.iter())
            .zip(self.masses.iter())
            .map(|((p, v), m)| (p, v, m))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut Position, &mut Velocity, &mut Mass)> {
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

    pub fn clear(&mut self) {
        self.positions.clear();
        self.velocities.clear();
        self.masses.clear();
    }

    pub fn reserve(&mut self, n: usize) {
        self.positions.reserve(n);
        self.velocities.reserve(n);
        self.masses.reserve(n);
    }
}

pub trait ParticleFactory {
    fn create(&mut self, n: usize, particles: &mut Particles);
}

pub struct GeneratorFactory {
    pub position_generator: Box<dyn Generator<Position>>,
    pub velocity_generator: Box<dyn Generator<Velocity>>,
    pub mass_generator: Box<dyn Generator<Mass>>,
}

impl GeneratorFactory {
    pub fn new(
        position_generator: Box<dyn Generator<Position>>,
        velocity_generator: Box<dyn Generator<Velocity>>,
        mass_generator: Box<dyn Generator<Mass>>,
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
            .generate_n(n, &mut particles.positions);
        self.velocity_generator
            .generate_n(n, &mut particles.velocities);
        self.mass_generator.generate_n(n, &mut particles.masses);
    }
}
