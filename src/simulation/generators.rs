use std::f64::consts::PI;

use nalgebra::Vector2;
use rand::Rng;
use rand_xorshift::XorShiftRng;

use super::{
    areas::{Disk, Point, Rect},
    color::Color,
    types::Scalar,
};

pub trait Generator<T> {
    fn generate(&mut self) -> T;

    fn generate_n(&mut self, n: usize, vec: &mut Vec<T>) {
        vec.reserve(n);
        // TODO parallelize
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
    min: Scalar,
    max: Scalar,
}

impl UniformGenerator {
    pub fn new(rng: XorShiftRng, min: Scalar, max: Scalar) -> Self {
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
        vec.reserve(n);
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
        vec.reserve(n);
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

pub struct RectGenerator {
    rect: Rect,
    rng: XorShiftRng,
}

impl RectGenerator {
    pub fn new(rect: Rect, rng: XorShiftRng) -> Self {
        Self { rect, rng }
    }
}

impl Generator<Vector2<Scalar>> for RectGenerator {
    fn generate(&mut self) -> Vector2<Scalar> {
        Vector2::new(
            self.rng.gen::<Scalar>() * self.rect.size.x + self.rect.position.x,
            self.rng.gen::<Scalar>() * self.rect.size.y + self.rect.position.y,
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

impl Generator<Vector2<Scalar>> for DiskGenerator {
    fn generate(&mut self) -> Vector2<Scalar> {
        let angle = self.rng.gen::<Scalar>() * 2.0 * PI;
        let radius = (self.rng.gen::<Scalar>() * self.disk.radius_squared).sqrt();
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
        vec.reserve(n);
        for ((r, g), (b, a)) in r
            .into_iter()
            .zip(g.into_iter())
            .zip(b.into_iter().zip(a.into_iter()))
        {
            vec.push(Color::new(r, g, b, a));
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

    pub fn hsv2rgb(h: Scalar, s: Scalar, v: Scalar) -> (Scalar, Scalar, Scalar) {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h {
            h if h < 60.0 => (c, x, 0.0),
            h if h < 120.0 => (x, c, 0.0),
            h if h < 180.0 => (0.0, c, x),
            h if h < 240.0 => (0.0, x, c),
            h if h < 300.0 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        (r + m, g + m, b + m)
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
        vec.reserve(n);
        for ((h, s), (v, a)) in h
            .into_iter()
            .zip(s.into_iter())
            .zip(v.into_iter().zip(a.into_iter()))
        {
            let (r, g, b) = Self::hsv2rgb(h, s, v);
            vec.push(Color::new(r, g, b, a));
        }
    }

    fn generate(&mut self) -> Color {
        let mut vec = Vec::with_capacity(1);
        self.generate_n(1, &mut vec);
        vec.pop().unwrap()
    }
}
