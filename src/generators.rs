use nalgebra::Vector2;
use rand::Rng;
use rand_xorshift::XorShiftRng;

use crate::areas::{Disk, Point, Rect};

pub trait Generator<T> {
    fn generate(&mut self, n: usize, vec: &mut Vec<T>);
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
    fn generate(&mut self, n: usize, vec: &mut Vec<T>) {
        for _ in 0..n {
            vec.push(self.value.clone());
        }
    }
}

pub struct UniformGenerator {
    rng: XorShiftRng,
    min: f32,
    max: f32,
}

impl UniformGenerator {
    pub fn new(rng: XorShiftRng, min: f32, max: f32) -> Self {
        Self { rng, min, max }
    }
}

impl Generator<f32> for UniformGenerator {
    fn generate(&mut self, n: usize, vec: &mut Vec<f32>) {
        for _ in 0..n {
            let value = self.rng.gen::<u32>() as f32 / std::u32::MAX as f32;
            vec.push(value * (self.max - self.min) + self.min);
        }
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
    fn generate(&mut self, n: usize, vec: &mut Vec<Vector2<f32>>) {
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);

        self.x_generator.generate(n, &mut x);
        self.y_generator.generate(n, &mut y);

        for (x, y) in x.into_iter().zip(y.into_iter()) {
            vec.push(Vector2::new(x, y));
        }
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
    fn generate(&mut self, n: usize, vec: &mut Vec<Vector2<f32>>) {
        let mut r = Vec::with_capacity(n);
        let mut theta = Vec::with_capacity(n);

        self.r_generator.generate(n, &mut r);
        self.theta_generator.generate(n, &mut theta);

        for (r, theta) in r.into_iter().zip(theta.into_iter()) {
            vec.push(Vector2::new(r * theta.cos(), r * theta.sin()));
        }
    }
}

impl Generator<Vector2<f32>> for Rect {
    fn generate(&mut self, n: usize, vec: &mut Vec<Vector2<f32>>) {
        for _ in 0..n {
            vec.push(Vector2::new(
                rand::random::<f32>() * self.size.x + self.position.x,
                rand::random::<f32>() * self.size.y + self.position.y,
            ));
        }
    }
}

impl Generator<Vector2<f32>> for Disk {
    fn generate(&mut self, n: usize, vec: &mut Vec<Vector2<f32>>) {
        for _ in 0..n {
            let angle = rand::random::<f32>() * 2. * std::f32::consts::PI;
            let radius = rand::random::<f32>().sqrt() * self.radius;
            vec.push(Vector2::new(
                radius * angle.cos() + self.position.x,
                radius * angle.sin() + self.position.y,
            ));
        }
    }
}

impl Generator<Vector2<f32>> for Point {
    fn generate(&mut self, n: usize, vec: &mut Vec<Vector2<f32>>) {
        for _ in 0..n {
            vec.push(self.position);
        }
    }
}
