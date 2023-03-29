use nalgebra::Vector2;
use sfml::graphics::{CircleShape, Color, RenderTarget, RenderWindow, Transformable};
use sfml::system::Vector2f;
use sfml::window::Event;

use crate::particle::Particle;
use crate::simulation::Simulation;

pub struct IridiumRenderer<'a> {
    pub window: RenderWindow,
    pub particle_shape: CircleShape<'a>,
}

impl<'a> IridiumRenderer<'a> {
    pub fn new(window: RenderWindow, particle_shape: CircleShape<'a>) -> Self {
        Self {
            window,
            particle_shape,
        }
    }

    // Convert simulation position to screen position
    pub fn sim2screen(&self, sim: &Simulation, position: Vector2<f32>) -> (i16, i16) {
        (
            position.x as i16,
            self.window.size().y as i16 - position.y as i16,
        )
    }

    pub fn render_particle(&mut self, simulation: &Simulation, particle: &Particle) {
        let (x, y) = self.sim2screen(&simulation, particle.position);
        // Position
        self.particle_shape
            .set_position(Vector2f::new(x as f32, y as f32));
        self.window.draw(&self.particle_shape);
    }

    pub fn render(&mut self, simulation: &Simulation) {
        self.window.clear(Color::BLACK);

        for particle in &simulation.particles {
            self.render_particle(simulation, particle);
        }

        self.window.display();
    }

    pub fn process_events(&mut self, simulation: &mut Simulation) {
        while let Some(event) = self.window.poll_event() {
            match event {
                Event::Closed => self.window.close(),
                Event::MouseButtonPressed {
                    button: sfml::window::mouse::Button::Left,
                    x,
                    y,
                    ..
                } => {
                    let angle = rand::random::<f32>() * 2. * std::f32::consts::PI;
                    let velocity = Vector2::new(angle.cos(), angle.sin()) * 1.;

                    simulation.particles.push(Particle::new(
                        Vector2::new(x as f32, y as f32),
                        velocity,
                        1.0,
                    ));
                }
                _ => {}
            }
        }
    }

    // Default loop for quick prototyping
    pub fn render_loop(&mut self, simulation: &mut Simulation) {
        let mut last_log = std::time::Instant::now();
        let mut frame_count = 0;
        let mut elapsed: f32;
        let log_delta = 1.0; // Log every second

        while self.window.is_open() {
            simulation.update();
            self.render(simulation);
            self.process_events(simulation);

            frame_count += 1;

            elapsed = last_log.elapsed().as_secs_f32();
            if elapsed >= log_delta {
                let frame_time_av = elapsed / frame_count as f32;

                println!(
                    "{} frames in {} s\n~{} ms/frame ({} fps)\n{} particles ({} µs/particle)\n",
                    frame_count,
                    elapsed,
                    frame_time_av * 1000.,
                    (1. / frame_time_av) as i32,
                    simulation.particles.len(),
                    (elapsed * 1_000_000.) / frame_count as f32
                );

                last_log = std::time::Instant::now();
                frame_count = 0;
            }
        }
    }
}
