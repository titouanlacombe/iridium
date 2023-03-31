use nalgebra::Vector2;
use sfml::graphics::{Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Vertex};
use sfml::system::Vector2f;
use sfml::window::{Event, Key};
use std::time::{Duration, Instant};

use crate::particle::Particle;
use crate::simulation::Simulation;

pub struct IridiumRenderer {
    pub window: RenderWindow,
    pub pos_buffer: Vec<Vertex>,
}

impl IridiumRenderer {
    pub fn new(window: RenderWindow) -> Self {
        Self {
            window,
            pos_buffer: Vec::new(),
        }
    }

    // Convert simulation position to screen position
    pub fn sim2screen(&self, sim: &Simulation, position: Vector2<f32>) -> Vector2f {
        Vector2f::new(position.x, self.window.size().y as f32 - position.y)
    }

    pub fn render(&mut self, simulation: &Simulation) {
        self.window.clear(Color::BLACK);

        self.pos_buffer.resize(
            simulation.particles.len(),
            Vertex::new(Vector2f::new(0., 0.), Color::WHITE, Vector2f::new(0., 0.)),
        );
        let mut i = 0;
        for particle in &simulation.particles {
            self.pos_buffer[i].position = self.sim2screen(simulation, particle.position);
            i += 1;
        }

        // Draw all particles
        self.window.draw_primitives(
            &self.pos_buffer,
            PrimitiveType::POINTS,
            &RenderStates::default(),
        );

        self.window.display();
    }

    pub fn process_events(&mut self, simulation: &mut Simulation) {
        while let Some(event) = self.window.poll_event() {
            match event {
                Event::Closed => self.window.close(),
                Event::KeyPressed { code, .. } => {
                    if code == Key::Escape {
                        self.window.close();
                    }
                }
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
        let log_delta = 1.0; // Log every second

        let mut last_log = Instant::now();
        let mut sim_elapsed = Duration::ZERO;
        let mut render_elapsed = Duration::ZERO;
        let mut events_elapsed = Duration::ZERO;
        let mut frame_count = 0;

        let mut log_elapsed;
        let mut t;

        while self.window.is_open() {
            t = Instant::now();
            simulation.update();
            sim_elapsed += t.elapsed();

            t = Instant::now();
            self.render(simulation);
            render_elapsed += t.elapsed();

            t = Instant::now();
            self.process_events(simulation);
            events_elapsed += t.elapsed();

            frame_count += 1;

            log_elapsed = last_log.elapsed().as_secs_f64();
            if log_elapsed >= log_delta {
                let frame_time_av = log_elapsed / frame_count as f64;
                let particle_count = simulation.particles.len();

                println!(
                    "\n{} steps in {:.2} s (~{:.2} fps)\n\
					{:.2} ms/step ({:.2} ms/sim, {:.2} ms/render, {:.2} ms/events)\n\
					{:.2e} particles ({:.2} µs/particle)\n",
                    frame_count,
                    log_elapsed,
                    1. / frame_time_av,
                    frame_time_av * 1000.,
                    sim_elapsed.as_secs_f64() * 1000. / frame_count as f64,
                    render_elapsed.as_secs_f64() * 1000. / frame_count as f64,
                    events_elapsed.as_secs_f64() * 1000. / frame_count as f64,
                    particle_count,
                    ((log_elapsed * 1e6) / frame_count as f64) / particle_count as f64,
                );

                last_log = Instant::now();
                sim_elapsed = Duration::ZERO;
                render_elapsed = Duration::ZERO;
                events_elapsed = Duration::ZERO;
                frame_count = 0;
            }
        }
    }
}
