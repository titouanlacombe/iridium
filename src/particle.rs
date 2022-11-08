use nalgebra::Vector2;

pub struct Particle {
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub mass: f32,
}

// TODO turn into interface
pub struct Area {}

pub struct Drain {
    pub area: Area,
    pub rate: f32,
}

// TODO turn into interface
pub struct ParticleFactory {}

pub struct Tap {
    pub p_factory: ParticleFactory,
    pub rate: f32,
}
