use std::f64::consts::PI;

use nalgebra::Vector2;
use rand::Rng;
use rand_pcg::Pcg64Mcg;

use super::{
    areas::{Disk, Point, Rect},
    color::Color,
    types::Scalar,
};

pub trait Generator<T> {
    fn generate(&mut self) -> T;

    fn generate_n(&mut self, n: usize, vec: &mut Vec<T>) {
        vec.reserve_exact(n);
        // TODO parallelize
        for _ in 0..n {
            vec.push(self.generate());
        }
    }
}

pub struct IterGenerator<T: Iterator> {
    pub iter: T,
}

impl<T: Iterator> IterGenerator<T> {
    pub fn new(iter: T) -> Self {
        Self { iter }
    }
}

impl<T: Iterator> Generator<T::Item> for IterGenerator<T> {
    fn generate(&mut self) -> T::Item {
        self.iter.next().expect("Iterator is empty")
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
    rng: Pcg64Mcg,
    min: Scalar,
    max: Scalar,
}

impl UniformGenerator {
    pub fn new(rng: Pcg64Mcg, min: Scalar, max: Scalar) -> Self {
        if min >= max {
            panic!("min must be less than max");
        }
        Self { rng, min, max }
    }
}

impl Generator<Scalar> for UniformGenerator {
    fn generate(&mut self) -> Scalar {
        self.rng.gen_range(self.min..self.max)
    }
}

pub struct Vector2Generator {
    pub x_generator: Box<dyn Generator<Scalar>>,
    pub y_generator: Box<dyn Generator<Scalar>>,
}

impl Vector2Generator {
    pub fn new(
        x_generator: Box<dyn Generator<Scalar>>,
        y_generator: Box<dyn Generator<Scalar>>,
    ) -> Self {
        Self {
            x_generator,
            y_generator,
        }
    }
}

impl Generator<Vector2<Scalar>> for Vector2Generator {
    fn generate_n(&mut self, n: usize, vec: &mut Vec<Vector2<Scalar>>) {
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);

        self.x_generator.generate_n(n, &mut x);
        self.y_generator.generate_n(n, &mut y);

        // TODO parallelize
        vec.reserve_exact(n);
        for (x, y) in x.into_iter().zip(y.into_iter()) {
            vec.push(Vector2::new(x, y));
        }
    }

    fn generate(&mut self) -> Vector2<Scalar> {
        let mut vec = Vec::with_capacity(1);
        self.generate_n(1, &mut vec);
        vec[0]
    }
}

pub struct Vector2PolarGenerator {
    pub r_generator: Box<dyn Generator<Scalar>>,
    pub theta_generator: Box<dyn Generator<Scalar>>,
}

impl Vector2PolarGenerator {
    pub fn new(
        r_generator: Box<dyn Generator<Scalar>>,
        theta_generator: Box<dyn Generator<Scalar>>,
    ) -> Self {
        Self {
            r_generator,
            theta_generator,
        }
    }
}

impl Generator<Vector2<Scalar>> for Vector2PolarGenerator {
    fn generate_n(&mut self, n: usize, vec: &mut Vec<Vector2<Scalar>>) {
        let mut r = Vec::with_capacity(n);
        let mut theta = Vec::with_capacity(n);

        self.r_generator.generate_n(n, &mut r);
        self.theta_generator.generate_n(n, &mut theta);

        // TODO parallelize
        vec.reserve_exact(n);
        for (r, theta) in r.into_iter().zip(theta.into_iter()) {
            vec.push(Vector2::new(r * theta.cos(), r * theta.sin()));
        }
    }

    fn generate(&mut self) -> Vector2<Scalar> {
        let mut vec = Vec::with_capacity(1);
        self.generate_n(1, &mut vec);
        vec[0]
    }
}

pub struct RandomRectPointGenerator {
    rect: Rect,
    rng: Pcg64Mcg,
}

impl RandomRectPointGenerator {
    pub fn new(rect: Rect, rng: Pcg64Mcg) -> Self {
        Self { rect, rng }
    }
}

impl Generator<Vector2<Scalar>> for RandomRectPointGenerator {
    fn generate(&mut self) -> Vector2<Scalar> {
        Vector2::new(
            self.rng.gen::<Scalar>() * self.rect.size.x + self.rect.position.x,
            self.rng.gen::<Scalar>() * self.rect.size.y + self.rect.position.y,
        )
    }
}

pub struct UniformRectPointsGenerator {
    rect: Rect,
}

impl UniformRectPointsGenerator {
    pub fn new(rect: Rect) -> Self {
        Self { rect }
    }
}

impl Generator<Vector2<Scalar>> for UniformRectPointsGenerator {
    fn generate(&mut self) -> Vector2<Scalar> {
        self.rect.center()
    }

    fn generate_n(&mut self, n: usize, vec: &mut Vec<Vector2<Scalar>>) {
        vec.reserve_exact(n);

        // TODO area::area() ?
        let area_per_point = (self.rect.size.x * self.rect.size.y) / n as f64;
        let distance_increment = area_per_point.sqrt();

        let nx = (self.rect.size.x / distance_increment).floor() as usize;
        let ny = (self.rect.size.y / distance_increment).floor() as usize;

        let dx = self.rect.size.x / (nx + 1) as f64;
        let dy = self.rect.size.y / (ny + 1) as f64;

        // Distribute points in the grid
        for i in 0..nx {
            for j in 0..ny {
                vec.push(Vector2::new(
                    self.rect.position.x + i as f64 * dx + dx / 2.0,
                    self.rect.position.y + j as f64 * dy + dy / 2.0,
                ));
            }
        }

        // Handle remainders
        let remainder = n - nx * ny;
        if remainder > 0 {
            let dx_remainder = self.rect.size.x / remainder as f64;
            for i in 0..remainder {
                vec.push(Vector2::new(
                    self.rect.position.x + i as f64 * dx_remainder + dx_remainder / 2.0,
                    self.rect.position.y + self.rect.size.y - dy / 2.0,
                ));
            }
        }
    }
}

pub struct RandomDiskPointGenerator {
    disk: Disk,
    rng: Pcg64Mcg,
}

impl RandomDiskPointGenerator {
    pub fn new(disk: Disk, rng: Pcg64Mcg) -> Self {
        Self { disk, rng }
    }
}

impl Generator<Vector2<Scalar>> for RandomDiskPointGenerator {
    fn generate(&mut self) -> Vector2<Scalar> {
        let angle = self.rng.gen::<Scalar>() * 2.0 * PI;
        let radius = (self.rng.gen::<Scalar>() * self.disk.radius_squared).sqrt();
        Vector2::new(
            self.disk.position.x + radius * angle.cos(),
            self.disk.position.y + radius * angle.sin(),
        )
    }
}

pub struct UniformDiskPointsGenerator {
    disk: Disk,
}

impl UniformDiskPointsGenerator {
    pub fn new(disk: Disk) -> Self {
        Self { disk }
    }
}

impl Generator<Vector2<Scalar>> for UniformDiskPointsGenerator {
    fn generate(&mut self) -> Vector2<Scalar> {
        self.disk.position
    }

    fn generate_n(&mut self, n: usize, vec: &mut Vec<Vector2<Scalar>>) {
        vec.reserve_exact(n);

        let total_area = PI * self.disk.radius_squared;
        let area_per_point = total_area / n as f64;
        let distance_increment = area_per_point.sqrt();

        let nr = (self.disk.radius_squared.sqrt() / distance_increment).ceil() as usize;
        let mut placed = 0;
        for i in 0..nr {
            let r = (i as f64 + 0.5) * distance_increment;
            let circumference = 2.0 * PI * r;
            let points_on_this_radius =
                std::cmp::min(n - placed, (circumference / distance_increment) as usize);
            let angle_increment = 2.0 * PI / points_on_this_radius as f64;
            for j in 0..points_on_this_radius {
                let angle = j as f64 * angle_increment;
                let x = r * angle.cos();
                let y = r * angle.sin();
                vec.push(Vector2::new(x, y) + self.disk.position);
            }
            placed += points_on_this_radius;
        }
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

impl Generator<Vector2<Scalar>> for PointGenerator {
    fn generate(&mut self) -> Vector2<Scalar> {
        self.point.position
    }
}

pub struct RGBAGenerator {
    pub r_generator: Box<dyn Generator<Scalar>>,
    pub g_generator: Box<dyn Generator<Scalar>>,
    pub b_generator: Box<dyn Generator<Scalar>>,
    pub a_generator: Box<dyn Generator<Scalar>>,
}

impl RGBAGenerator {
    pub fn new(
        r_generator: Box<dyn Generator<Scalar>>,
        g_generator: Box<dyn Generator<Scalar>>,
        b_generator: Box<dyn Generator<Scalar>>,
        a_generator: Box<dyn Generator<Scalar>>,
    ) -> Self {
        Self {
            r_generator,
            g_generator,
            b_generator,
            a_generator,
        }
    }
}

impl Generator<Color> for RGBAGenerator {
    fn generate_n(&mut self, n: usize, vec: &mut Vec<Color>) {
        let mut r = Vec::with_capacity(n);
        let mut g = Vec::with_capacity(n);
        let mut b = Vec::with_capacity(n);
        let mut a = Vec::with_capacity(n);

        self.r_generator.generate_n(n, &mut r);
        self.g_generator.generate_n(n, &mut g);
        self.b_generator.generate_n(n, &mut b);
        self.a_generator.generate_n(n, &mut a);

        // TODO parallelize
        vec.reserve_exact(n);
        for ((r, g), (b, a)) in r
            .into_iter()
            .zip(g.into_iter())
            .zip(b.into_iter().zip(a.into_iter()))
        {
            vec.push(Color::from_rgba(r, g, b, a));
        }
    }

    fn generate(&mut self) -> Color {
        let mut vec = Vec::with_capacity(1);
        self.generate_n(1, &mut vec);
        vec.pop().unwrap()
    }
}

pub struct HSVAGenerator {
    pub h_generator: Box<dyn Generator<Scalar>>,
    pub s_generator: Box<dyn Generator<Scalar>>,
    pub v_generator: Box<dyn Generator<Scalar>>,
    pub a_generator: Box<dyn Generator<Scalar>>,
}

impl HSVAGenerator {
    pub fn new(
        h_generator: Box<dyn Generator<Scalar>>,
        s_generator: Box<dyn Generator<Scalar>>,
        v_generator: Box<dyn Generator<Scalar>>,
        a_generator: Box<dyn Generator<Scalar>>,
    ) -> Self {
        Self {
            h_generator,
            s_generator,
            v_generator,
            a_generator,
        }
    }
}

impl Generator<Color> for HSVAGenerator {
    fn generate_n(&mut self, n: usize, vec: &mut Vec<Color>) {
        let mut h = Vec::with_capacity(n);
        let mut s = Vec::with_capacity(n);
        let mut v = Vec::with_capacity(n);
        let mut a = Vec::with_capacity(n);

        self.h_generator.generate_n(n, &mut h);
        self.s_generator.generate_n(n, &mut s);
        self.v_generator.generate_n(n, &mut v);
        self.a_generator.generate_n(n, &mut a);

        // TODO parallelize
        vec.reserve_exact(n);
        for ((h, s), (v, a)) in h
            .into_iter()
            .zip(s.into_iter())
            .zip(v.into_iter().zip(a.into_iter()))
        {
            vec.push(Color::from_hsva(h, s, v, a));
        }
    }

    fn generate(&mut self) -> Color {
        let mut vec = Vec::with_capacity(1);
        self.generate_n(1, &mut vec);
        vec.pop().unwrap()
    }
}
