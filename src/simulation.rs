use crate::particle::{Consumer, Emitter, Particle};

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
            let mut p_pos = &mut particle.position;
            let mut p_vel = &mut particle.velocity;

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
            let mut p_pos = &mut particle.position;

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
    pub limit: LimitCond,
}

impl Simulation {
    pub fn new(
        particles: Vec<Particle>,
        emitters: Vec<Emitter>,
        consumers: Vec<Consumer>,
        limit: LimitCond,
    ) -> Self {
        Simulation {
            particles,
            emitters,
            consumers,
            limit,
        }
    }

    pub fn new_empty(limit: LimitCond) -> Simulation {
        Simulation::new(Vec::new(), Vec::new(), Vec::new(), limit)
    }

    pub fn update(&mut self) {
        let dt = 1.0;

        // Emit new particles
        for emitter in &self.emitters {
            let limit = (emitter.rate * dt) as usize;

            for _ in 0..limit {
                self.particles.push(emitter.p_factory.new());
            }
        }

        // Consume particles
        for consumer in &self.consumers {
            let mut to_remove = Vec::new();
            let limit = (consumer.rate * dt) as usize;

            for (i, particle) in self.particles.iter_mut().enumerate() {
                if consumer.area.contains(particle.position) {
                    to_remove.push(i);

                    if to_remove.len() >= limit {
                        break;
                    }
                }
            }

            for i in to_remove {
                self.particles.swap_remove(i);
            }
        }

        // Update particles
        // List of changes to not modify while iterating
        let mut to_add: Vec<Particle> = Vec::new();
        let mut to_remove: Vec<usize> = Vec::new();

        for (i, particle) in self.particles.iter_mut().enumerate() {
            // Update position
            particle.position += particle.velocity * dt;

            // Check limits
            limit_update(&self.limit, i, particle, &mut to_remove)
        }

        // Remove double to_remove entries
        // to_remove.sort();
        // to_remove.dedup();

        for i in to_remove {
            self.particles.swap_remove(i);
        }

        self.particles.append(&mut to_add);
    }
}
