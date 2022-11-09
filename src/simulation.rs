use nalgebra::Vector2;

use crate::particle::{Drain, Particle, Tap, Updatable};

pub enum LimitCond {
    // infinite space
    // no effect
    None,

    // wall at position, with size and restitution
    // will bounce objects
    Wall(Vector2<f32>, f32, f32, f32),

    // loop at position, with size
    // object will be teleported to the other side
    Loop(Vector2<f32>, f32, f32),

    // void at position, with size
    // object will be destroyed on exit
    Void(Vector2<f32>, f32, f32),
}

pub struct Simulation {
    pub particles: Vec<Particle>,
    pub taps: Vec<Tap>,
    pub drains: Vec<Drain>,
    pub limit: LimitCond,
}

impl Simulation {
    pub fn new(
        particles: Vec<Particle>,
        taps: Vec<Tap>,
        drains: Vec<Drain>,
        limit: LimitCond,
    ) -> Self {
        Simulation {
            particles,
            taps,
            drains,
            limit,
        }
    }

    pub fn new_empty(limit: LimitCond) -> Simulation {
        Simulation::new(Vec::new(), Vec::new(), Vec::new(), limit)
    }

    pub fn update(&mut self) {
        let dt = 1.0;

        // --- Update objects ---
        for i in 0..self.particles.len() {
            self.particles[i].update(dt);
        }

        for i in 0..self.taps.len() {
            self.taps[i].update(dt, &mut self.particles);
        }

        for i in 0..self.drains.len() {
            self.drains[i].update(dt, &mut self.particles);
        }

        // --- Update limit conditions ---
        self.limit_update();
    }

    fn limit_update(&mut self) {
        match self.limit {
            LimitCond::None => {}
            LimitCond::Wall(l_pos, width, height, restitution) => {
                let x_max = l_pos.x + width;
                let y_max = l_pos.y + height;

                for i in 0..self.particles.len() {
                    let p = &mut self.particles[i];
                    let mut p_pos = &mut p.position;
                    let mut p_vel = &mut p.velocity;

                    if p_pos.x < l_pos.x {
                        p_pos.x = l_pos.x;
                        p_vel.x = -p_vel.x * restitution;
                    } else if p_pos.x > x_max {
                        p_pos.x = x_max;
                        p_vel.x = -p_vel.x * restitution;
                    }

                    if p_pos.y < l_pos.y {
                        p_pos.y = l_pos.y;
                        p_vel.y = -p_vel.y * restitution;
                    } else if p_pos.y > y_max {
                        p_pos.y = y_max;
                        p_vel.y = -p_vel.y * restitution;
                    }
                }
            }
            LimitCond::Loop(l_pos, width, height) => {
                let x_max = l_pos.x + width;
                let y_max = l_pos.y + height;

                for i in 0..self.particles.len() {
                    let mut p_pos = &mut self.particles[i].position;

                    if p_pos.x < l_pos.x {
                        p_pos.x = x_max;
                    } else if p_pos.x > x_max {
                        p_pos.x = l_pos.x;
                    }

                    if p_pos.y < l_pos.y {
                        p_pos.y = y_max;
                    } else if p_pos.y > y_max {
                        p_pos.y = l_pos.y;
                    }
                }
            }
            LimitCond::Void(l_pos, width, height) => {
                let x_max = l_pos.x + width;
                let y_max = l_pos.y + height;
                let mut to_remove: Vec<usize> = Vec::new();

                for i in 0..self.particles.len() {
                    let p_pos = &self.particles[i].position;

                    if p_pos.x < l_pos.x || p_pos.x > x_max || p_pos.y < l_pos.y || p_pos.y > y_max
                    {
                        to_remove.push(i);
                    }
                }

                // Fast remove
                for i in to_remove {
                    self.particles.swap_remove(i);
                }
            }
        }
    }
}
