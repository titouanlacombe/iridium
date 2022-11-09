use nalgebra::{Norm, Vector2};

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

pub trait Area {
    fn contains(&self, position: Vector2<f32>) -> bool;
    fn rand(&self) -> Vector2<f32>;
}

pub struct Rect {
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,
}

impl Area for Rect {
    fn contains(&self, position: Vector2<f32>) -> bool {
        position.x >= self.position.x
            && position.x <= self.position.x + self.size.x
            && position.y >= self.position.y
            && position.y <= self.position.y + self.size.y
    }

    fn rand(&self) -> Vector2<f32> {
        Vector2::new(
            self.position.x + rand::random::<f32>() * self.size.x,
            self.position.y + rand::random::<f32>() * self.size.y,
        )
    }
}

pub struct Disk {
    pub position: Vector2<f32>,
    pub radius: f32,
}

impl Area for Disk {
    fn contains(&self, position: Vector2<f32>) -> bool {
        (position - self.position).norm() <= self.radius
    }

    fn rand(&self) -> Vector2<f32> {
        let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;

        // For a uniform distribution, we need to square root the random number
        let radius = rand::random::<f32>().sqrt() * self.radius;
        self.position + Vector2::new(radius * angle.cos(), radius * angle.sin())
    }
}

pub struct Consumer {
    pub area: Box<dyn Area>,
    pub rate: f32,
}

impl Consumer {
    pub fn new(area: Box<dyn Area>, rate: f32) -> Consumer {
        Consumer { area, rate }
    }
}

impl Updatable for Consumer {
    fn update(&mut self, dt: f32, particles: &mut Vec<Particle>) {
        let limit = (self.rate * dt) as usize;
        let mut to_remove = Vec::new();

        for i in 0..particles.len() {
            if self.area.contains(particles[i].position) {
                to_remove.push(i);

                if to_remove.len() >= limit {
                    break;
                }
            }
        }

        for i in to_remove {
            particles.swap_remove(i);
        }
    }
}

fn rand_range(min: f32, max: f32) -> f32 {
    min + (max - min) * rand::random::<f32>()
}

pub trait ParticleFactory {
    fn new(&self) -> Particle;
}

pub struct RandomFactory {
    pub area: Box<dyn Area>,
    pub velocity_min: f32,
    pub velocity_max: f32,
    pub velocity_angle_min: f32,
    pub velocity_angle_max: f32,

    pub mass_min: f32,
    pub mass_max: f32,
}

impl RandomFactory {
    pub fn new(
        area: Box<dyn Area>,
        velocity_min: f32,
        velocity_max: f32,
        velocity_angle_min: f32,
        velocity_angle_max: f32,
        mass_min: f32,
        mass_max: f32,
    ) -> RandomFactory {
        RandomFactory {
            area,
            velocity_min,
            velocity_max,
            velocity_angle_min,
            velocity_angle_max,
            mass_min,
            mass_max,
        }
    }
}

impl ParticleFactory for RandomFactory {
    fn new(&self) -> Particle {
        let position = self.area.rand();

        let velocity_magn = rand_range(self.velocity_min, self.velocity_max);
        let velocity_angle = rand_range(self.velocity_angle_min, self.velocity_angle_max);
        let velocity = Vector2::new(
            velocity_magn * velocity_angle.cos(),
            velocity_magn * velocity_angle.sin(),
        );

        let mass = rand_range(self.mass_min, self.mass_max);

        Particle::new(position, velocity, mass)
    }
}

pub struct Emitter {
    pub p_factory: Box<dyn ParticleFactory>,
    pub rate: f32,
}

impl Emitter {
    pub fn new(p_factory: Box<dyn ParticleFactory>, rate: f32) -> Emitter {
        Emitter { p_factory, rate }
    }
}

impl Updatable for Emitter {
    fn update(&mut self, dt: f32, particles: &mut Vec<Particle>) {
        let limit = (self.rate * dt) as usize;

        for _ in 0..limit {
            particles.push(self.p_factory.new());
        }
    }
}
