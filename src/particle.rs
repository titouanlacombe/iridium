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

    pub fn update(&mut self) {
        self.position += self.velocity;
    }
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
