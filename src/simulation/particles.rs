use super::{
    color::Color,
    generators::Generator,
    types::{Mass, Position, Velocity},
};

pub struct Particles {
    pub positions: Vec<Position>,
    pub velocities: Vec<Velocity>,
    pub masses: Vec<Mass>,
    // inv_masses[i] == 1.0 / masses[i]. Keep in sync on every masses write.
    pub inv_masses: Vec<Mass>,
    pub colors: Vec<Color>,
}

impl Particles {
    pub fn new(
        positions: Vec<Position>,
        velocities: Vec<Velocity>,
        masses: Vec<Mass>,
        colors: Vec<Color>,
    ) -> Self {
        let inv_masses = masses.iter().map(|mass| 1.0 / *mass).collect();
        Self {
            positions,
            velocities,
            masses,
            inv_masses,
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
        self.inv_masses.swap_remove(i);
        self.colors.swap_remove(i);
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.velocities.clear();
        self.masses.clear();
        self.inv_masses.clear();
        self.colors.clear();
    }

    pub fn reserve_exact(&mut self, n: usize) {
        self.positions.reserve_exact(n);
        self.velocities.reserve_exact(n);
        self.masses.reserve_exact(n);
        self.inv_masses.reserve_exact(n);
        self.colors.reserve_exact(n);
    }

    pub fn shrink_to_fit(&mut self) {
        self.positions.shrink_to_fit();
        self.velocities.shrink_to_fit();
        self.masses.shrink_to_fit();
        self.inv_masses.shrink_to_fit();
        self.colors.shrink_to_fit();
    }

    pub fn copy_from_indexes(&mut self, indexes: &Vec<usize>, particles: &Particles) {
        self.clear();
        self.reserve_exact(indexes.len());
        indexes.iter().for_each(|&i| {
            self.positions.push(particles.positions[i]);
            self.velocities.push(particles.velocities[i]);
            self.masses.push(particles.masses[i]);
            self.inv_masses.push(particles.inv_masses[i]);
            self.colors.push(particles.colors[i]);
        });
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
        let _span = tracy_client::span!("Particle Factory");
        self.position_generator
            .generate_n(n, &mut particles.positions);
        self.velocity_generator
            .generate_n(n, &mut particles.velocities);
        self.mass_generator.generate_n(n, &mut particles.masses);
        let start = particles.masses.len() - n;
        particles.inv_masses.reserve_exact(n);
        particles.masses[start..].iter().for_each(|mass| {
            particles.inv_masses.push(1.0 / *mass);
        });
        self.color_generator.generate_n(n, &mut particles.colors);
    }
}
