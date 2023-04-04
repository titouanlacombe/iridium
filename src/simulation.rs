use nalgebra::Vector2;

use crate::{
    forces::{UniformDrag, UniformGravity},
    particle::{Consumer, Emitter, Particle},
};

pub enum LimitCond {
    // infinite space
    // no effect
    None,

    // wall at position, with size and restitution
    // will bounce objects
    Wall(f32, f32, f32, f32, f32),

    // loop at position, with size
    // object will be teleported to the other side
    Loop(f32, f32, f32, f32),

    // void at position, with size
    // object will be destroyed on exit
    Void(f32, f32, f32, f32),
}

fn limit_update(limit: &LimitCond, i: usize, particle: &mut Particle, to_remove: &mut Vec<usize>) {
    match limit {
        LimitCond::None => {}
        LimitCond::Wall(x_min, y_min, x_max, y_max, restitution) => {
            let p_pos = &mut particle.position;
            let p_vel = &mut particle.velocity;

            if p_pos.x < *x_min {
                p_pos.x = *x_min;
                p_vel.x = -p_vel.x * restitution;
            } else if p_pos.x > *x_max {
                p_pos.x = *x_max;
                p_vel.x = -p_vel.x * restitution;
            }

            if p_pos.y < *y_min {
                p_pos.y = *y_min;
                p_vel.y = -p_vel.y * restitution;
            } else if p_pos.y > *y_max {
                p_pos.y = *y_max;
                p_vel.y = -p_vel.y * restitution;
            }
        }
        LimitCond::Loop(x_min, y_min, x_max, y_max) => {
            let p_pos = &mut particle.position;

            if p_pos.x < *x_min {
                p_pos.x = *x_max;
            } else if p_pos.x > *x_max {
                p_pos.x = *x_min;
            }

            if p_pos.y < *y_min {
                p_pos.y = *y_max;
            } else if p_pos.y > *y_max {
                p_pos.y = *y_min;
            }
        }
        LimitCond::Void(x_min, y_min, x_max, y_max) => {
            let p_pos = &mut particle.position;

            if p_pos.x < *x_min || p_pos.x > *x_max || p_pos.y < *y_min || p_pos.y > *y_max {
                to_remove.push(i);
            }
        }
    }
}

pub struct Simulation {
    pub particles: Vec<Particle>,
    pub emitters: Vec<Emitter>,
    pub consumers: Vec<Consumer>,

    // TODO: make these a Vec of Box<dyn Force>
    pub uniform_gravity: Option<UniformGravity>,
    pub uniform_drag: Option<UniformDrag>,

    pub limit: LimitCond,
}

impl Simulation {
    pub fn new(
        particles: Vec<Particle>,
        emitters: Vec<Emitter>,
        consumers: Vec<Consumer>,
        uniform_gravity: Option<UniformGravity>,
        uniform_drag: Option<UniformDrag>,
        limit: LimitCond,
    ) -> Self {
        Self {
            particles,
            emitters,
            consumers,
            uniform_gravity,
            uniform_drag,
            limit,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Emit new particles
        for emitter in &self.emitters {
            emitter.emit(&mut self.particles, dt);
        }

        // Consume particles
        for consumer in &self.consumers {
            consumer.consume(&mut self.particles, dt);
        }

        // Update particles
        // List of changes to not modify while iterating
        let mut to_add: Vec<Particle> = Vec::new();
        let mut to_remove: Vec<usize> = Vec::new();

        for (i, particle) in self.particles.iter_mut().enumerate() {
            let mut forces: Vector2<f32> = Vector2::new(0., 0.);

            // Apply forces
            if let Some(gravity) = &self.uniform_gravity {
                gravity.apply(particle, &mut forces);
            }

            if let Some(drag) = &self.uniform_drag {
                drag.apply(particle, &mut forces);
            }

            // Update particle
            particle.velocity += forces * dt / particle.mass;
            particle.position += particle.velocity * dt;

            // Check limits
            limit_update(&self.limit, i, particle, &mut to_remove)
        }

        for i in to_remove {
            self.particles.swap_remove(i);
        }

        self.particles.append(&mut to_add);
    }
}
