use super::{
    generators::Generator,
    types::{Color, Mass, Position, Velocity},
};

pub struct Particles {
    pub positions: Vec<Position>,
    pub velocities: Vec<Velocity>,
    pub masses: Vec<Mass>,
    pub colors: Vec<Color>,
}

impl Particles {
    pub fn new(
        positions: Vec<Position>,
        velocities: Vec<Velocity>,
        masses: Vec<Mass>,
        colors: Vec<Color>,
    ) -> Self {
        Self {
            positions,
            velocities,
            masses,
            colors,
        }
    }

    pub fn new_empty() -> Self {
        Self::new(Vec::new(), Vec::new(), Vec::new(), Vec::new())
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }

    pub fn swap_remove(&mut self, i: usize) {
        self.positions.swap_remove(i);
        self.velocities.swap_remove(i);
        self.masses.swap_remove(i);
        self.colors.swap_remove(i);
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.velocities.clear();
        self.masses.clear();
        self.colors.clear();
    }

    pub fn reserve(&mut self, n: usize) {
        self.positions.reserve(n);
        self.velocities.reserve(n);
        self.masses.reserve(n);
        self.colors.reserve(n);
    }
}

pub trait ParticleFactory {
    fn create(&mut self, n: usize, particles: &mut Particles);
}

pub struct GeneratorFactory {
    pub position_generator: Box<dyn Generator<Position>>,
    pub velocity_generator: Box<dyn Generator<Velocity>>,
    pub mass_generator: Box<dyn Generator<Mass>>,
    pub color_generator: Box<dyn Generator<Color>>,
}

impl GeneratorFactory {
    pub fn new(
        position_generator: Box<dyn Generator<Position>>,
        velocity_generator: Box<dyn Generator<Velocity>>,
        mass_generator: Box<dyn Generator<Mass>>,
        color_generator: Box<dyn Generator<Color>>,
    ) -> Self {
        Self {
            position_generator,
            velocity_generator,
            mass_generator,
            color_generator,
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
        self.color_generator.generate_n(n, &mut particles.colors);
    }
}
