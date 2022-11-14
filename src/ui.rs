use nalgebra::Vector2;

use crate::particle::Particle;
use crate::simulation::Simulation;

pub struct IridiumUI {
    pub running: bool,
    // pub renderer: IridiumRenderer,
}

impl IridiumUI {
    // pub fn new(renderer: IridiumRenderer) -> Self {
    //     Self {
    //         running: true,
    //         renderer,
    //     }
    // }

    // Convert simulation position to screen position
    pub fn sim2screen(&self, sim: &Simulation, position: Vector2<f32>) -> (i16, i16) {
        let height = 800;
        (position.x as i16, height as i16 - position.y as i16)
    }

    pub fn process_events(&mut self, simulation: &mut Simulation) {
        let (x, y) = (0, 0);

        // Spawn a particle on mouse click
        let angle = rand::random::<f32>() * 2. * std::f32::consts::PI;
        let velocity = Vector2::new(angle.cos(), angle.sin()) * 1.;

        simulation.particles.push(Particle::new(
            Vector2::new(x as f32, y as f32),
            velocity,
            1.0,
        ));
    }

    // Default loop for quick prototyping
    pub fn render_loop(&mut self, simulation: &mut Simulation) {
        let mut last_log = std::time::Instant::now();
        let mut frame_count = 0;
        let mut elapsed: f32;
        let log_delta = 1.0; // Log every second

        while self.running {
            simulation.update();

            frame_count += 1;

            elapsed = last_log.elapsed().as_secs_f32();
            if elapsed >= log_delta {
                let av_frame_time = elapsed / frame_count as f32;

                println!(
                    "{} frames in {} s\n~{} ms/frame ({} fps)\n{} particles ({} µs/particle)\n",
                    frame_count,
                    elapsed,
                    av_frame_time * 1000.,
                    (1. / av_frame_time) as i32,
                    simulation.particles.len(),
                    (elapsed * 1_000_000.) / frame_count as f32
                );

                last_log = std::time::Instant::now();
                frame_count = 0;
            }
        }
    }
}
