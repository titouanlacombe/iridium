use nalgebra::Vector2;

use crate::areas::{Disk, Point, Rect};

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

pub trait Generator<T> {
    fn generate(&self, n: usize) -> Vec<T>;
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

pub struct ConstantGenerator<T: Clone> {
    pub value: T,
}

impl<T: Clone> ConstantGenerator<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Clone> Generator<T> for ConstantGenerator<T> {
    fn generate(&self, n: usize) -> Vec<T> {
        (0..n).map(|_| self.value.clone()).collect()
    }
}

pub struct UniformGenerator {
    pub min: f32,
    pub max: f32,
}

impl UniformGenerator {
    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    fn range(min: f32, max: f32) -> f32 {
        min + (max - min) * rand::random::<f32>()
    }
}

impl Generator<f32> for UniformGenerator {
    fn generate(&self, n: usize) -> Vec<f32> {
        (0..n)
            .map(|_| UniformGenerator::range(self.min, self.max))
            .collect()
    }
}

pub struct Vector2Generator {
    pub x_generator: Box<dyn Generator<f32>>,
    pub y_generator: Box<dyn Generator<f32>>,
}

impl Vector2Generator {
    pub fn new(x_generator: Box<dyn Generator<f32>>, y_generator: Box<dyn Generator<f32>>) -> Self {
        Self {
            x_generator,
            y_generator,
        }
    }
}

impl Generator<Vector2<f32>> for Vector2Generator {
    fn generate(&self, n: usize) -> Vec<Vector2<f32>> {
        let x = self.x_generator.generate(n);
        let y = self.y_generator.generate(n);

        x.into_iter()
            .zip(y.into_iter())
            .map(|(x, y)| Vector2::new(x, y))
            .collect()
    }
}

pub struct Vector2PolarGenerator {
    pub r_generator: Box<dyn Generator<f32>>,
    pub theta_generator: Box<dyn Generator<f32>>,
}

impl Vector2PolarGenerator {
    pub fn new(
        r_generator: Box<dyn Generator<f32>>,
        theta_generator: Box<dyn Generator<f32>>,
    ) -> Self {
        Self {
            r_generator,
            theta_generator,
        }
    }
}

impl Generator<Vector2<f32>> for Vector2PolarGenerator {
    fn generate(&self, n: usize) -> Vec<Vector2<f32>> {
        let r = self.r_generator.generate(n);
        let theta = self.theta_generator.generate(n);

        r.into_iter()
            .zip(theta.into_iter())
            .map(|(r, theta)| Vector2::new(r * theta.cos(), r * theta.sin()))
            .collect()
    }
}

impl Generator<Vector2<f32>> for Rect {
    fn generate(&self, n: usize) -> Vec<Vector2<f32>> {
        let mut positions = Vec::with_capacity(n);
        for _ in 0..n {
            positions.push(Vector2::new(
                rand::random::<f32>() * self.size.x + self.position.x,
                rand::random::<f32>() * self.size.y + self.position.y,
            ));
        }
        positions
    }
}

impl Generator<Vector2<f32>> for Disk {
    fn generate(&self, n: usize) -> Vec<Vector2<f32>> {
        let mut positions = Vec::with_capacity(n);
        for _ in 0..n {
            let angle = rand::random::<f32>() * 2. * std::f32::consts::PI;
            let radius = rand::random::<f32>().sqrt() * self.radius;
            positions.push(Vector2::new(
                radius * angle.cos() + self.position.x,
                radius * angle.sin() + self.position.y,
            ));
        }
        positions
    }
}

impl Generator<Vector2<f32>> for Point {
    fn generate(&self, n: usize) -> Vec<Vector2<f32>> {
        (0..n).map(|_| self.position).collect()
    }
}
