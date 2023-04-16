use nalgebra::Vector2;
use rand::Rng;
use rand_xorshift::XorShiftRng;

use crate::areas::{Disk, Point, Rect};

pub trait Generator<T> {
    fn generate(&mut self) -> T;

    fn generate_n(&mut self, n: usize, vec: &mut Vec<T>) {
        vec.reserve(n);
        for _ in 0..n {
            vec.push(self.generate());
        }
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
    fn generate(&mut self) -> T {
        self.value.clone()
    }
}

pub struct UniformGenerator {
    rng: XorShiftRng,
    min: f32,
    max: f32,
}

impl UniformGenerator {
    pub fn new(rng: XorShiftRng, min: f32, max: f32) -> Self {
        if min >= max {
            panic!("min must be less than max");
        }
        Self { rng, min, max }
    }
}

impl Generator<f32> for UniformGenerator {
    fn generate(&mut self) -> f32 {
        self.rng.gen_range(self.min..self.max)
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
    fn generate_n(&mut self, n: usize, vec: &mut Vec<Vector2<f32>>) {
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);

        self.x_generator.generate_n(n, &mut x);
        self.y_generator.generate_n(n, &mut y);

        vec.reserve(n);
        for (x, y) in x.into_iter().zip(y.into_iter()) {
            vec.push(Vector2::new(x, y));
        }
    }

    fn generate(&mut self) -> Vector2<f32> {
        let mut vec = Vec::with_capacity(1);
        self.generate_n(1, &mut vec);
        vec[0]
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
    fn generate_n(&mut self, n: usize, vec: &mut Vec<Vector2<f32>>) {
        let mut r = Vec::with_capacity(n);
        let mut theta = Vec::with_capacity(n);

        self.r_generator.generate_n(n, &mut r);
        self.theta_generator.generate_n(n, &mut theta);

        vec.reserve(n);
        for (r, theta) in r.into_iter().zip(theta.into_iter()) {
            vec.push(Vector2::new(r * theta.cos(), r * theta.sin()));
        }
    }

    fn generate(&mut self) -> Vector2<f32> {
        let mut vec = Vec::with_capacity(1);
        self.generate_n(1, &mut vec);
        vec[0]
    }
}

pub struct RectGenerator {
    rect: Rect,
    rng: XorShiftRng,
}

impl RectGenerator {
    pub fn new(rect: Rect, rng: XorShiftRng) -> Self {
        Self { rect, rng }
    }
}

impl Generator<Vector2<f32>> for RectGenerator {
    fn generate(&mut self) -> Vector2<f32> {
        Vector2::new(
            self.rng.gen::<f32>() * self.rect.size.x + self.rect.position.x,
            self.rng.gen::<f32>() * self.rect.size.y + self.rect.position.y,
        )
    }
}

pub struct DiskGenerator {
    disk: Disk,
    rng: XorShiftRng,
}

impl DiskGenerator {
    pub fn new(disk: Disk, rng: XorShiftRng) -> Self {
        Self { disk, rng }
    }
}

impl Generator<Vector2<f32>> for DiskGenerator {
    fn generate(&mut self) -> Vector2<f32> {
        let angle = self.rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
        let radius = self.rng.gen::<f32>().sqrt() * self.disk.radius;
        Vector2::new(
            self.disk.position.x + radius * angle.cos(),
            self.disk.position.y + radius * angle.sin(),
        )
    }
}

pub struct PointGenerator {
    point: Point,
}

impl PointGenerator {
    pub fn new(point: Point) -> Self {
        Self { point }
    }
}

impl Generator<Vector2<f32>> for PointGenerator {
    fn generate(&mut self) -> Vector2<f32> {
        self.point.position
    }
}
