use nalgebra::Vector2;

pub struct Particle {
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub mass: f32,
}

impl Particle {
    pub fn new(position: Vector2<f32>, velocity: Vector2<f32>, mass: f32) -> Particle {
        Particle {
            position,
            velocity,
            mass,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.position += self.velocity * dt;
    }
}

pub trait Updatable {
    fn update(&mut self, dt: f32, particles: &mut Vec<Particle>);
}

// TODO turn into interface
pub struct Area {}

pub struct Drain {
    pub area: Area,
    pub rate: f32,
}

impl Drain {
    pub fn new(area: Area, rate: f32) -> Drain {
        Drain { area, rate }
    }
}

impl Updatable for Drain {
    fn update(&mut self, dt: f32, particles: &mut Vec<Particle>) {}
}

// TODO turn into interface
pub struct ParticleFactory {}

pub struct Tap {
    pub p_factory: ParticleFactory,
    pub rate: f32,
}

impl Tap {
    pub fn new(p_factory: ParticleFactory, rate: f32) -> Tap {
        Tap { p_factory, rate }
    }
}

impl Updatable for Tap {
    fn update(&mut self, dt: f32, particles: &mut Vec<Particle>) {}
}
