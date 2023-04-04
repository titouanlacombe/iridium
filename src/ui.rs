use nalgebra::Vector2;
use sfml::graphics::{Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Vertex};
use sfml::system::Vector2f;
use sfml::window::{Event, Key};
use std::time::{Duration, Instant};

use crate::particle::{ParticleFactory, Point, RandomFactory};
use crate::simulation::Simulation;

pub struct IridiumRenderer {
    pub window: RenderWindow,
    pub simulation: Simulation,

    pub screen_size: Vector2<f32>,
    pub pos_buffer: Vec<Vertex>,
}

impl IridiumRenderer {
    pub fn new(window: RenderWindow, simulation: Simulation) -> Self {
        Self {
            window,
            simulation,
            screen_size: Vector2::new(0., 0.),
            pos_buffer: Vec::new(),
        }
    }

    pub fn update_screen_size(&mut self) {
        let tmp = self.window.size();
        self.screen_size.x = tmp.x as f32;
        self.screen_size.y = tmp.y as f32;
    }

    // Convert between simulation and screen coordinates
    pub fn sim2screen(&self, position: Vector2<f32>) -> Vector2f {
        Vector2f::new(position.x, self.screen_size.y - position.y)
    }

    pub fn screen2sim(&self, position: Vector2f) -> Vector2<f32> {
        Vector2::new(position.x, self.screen_size.y - position.y)
    }

    pub fn render(&mut self) {
        // Cache current screen size
        self.update_screen_size();

        // Resize particle buffer
        let vertex = Vertex::new(Vector2f::new(0., 0.), Color::WHITE, Vector2f::new(0., 0.));
        self.pos_buffer
            .resize(self.simulation.particles.len(), vertex);

        // Update position buffer
        let mut i = 0;
        for particle in &self.simulation.particles {
            self.pos_buffer[i].position = self.sim2screen(particle.position);
            i += 1;
        }

        // Clear screen
        self.window.clear(Color::BLACK);

        // Draw buffer
        self.window.draw_primitives(
            &self.pos_buffer,
            PrimitiveType::POINTS,
            &RenderStates::default(),
        );

        // Display
        self.window.display();
    }

    pub fn process_events(&mut self) {
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
                    let pfactory = RandomFactory::new(
                        Box::new(Point {
                            position: self.screen2sim(Vector2f::new(x as f32, y as f32)),
                        }),
                        0.,
                        1.,
                        0.,
                        2. * std::f32::consts::PI,
                        1.,
                        1.,
                    );

                    for _ in 0..1000 {
                        self.simulation.particles.push(pfactory.create());
                    }
                }
                _ => {}
            }
        }
    }

    // Default loop for quick prototyping
    // TODO move to default simulation runner (ui takes runner as argument)
    pub fn render_loop(&mut self) {
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
            self.simulation.update(1.);
            sim_elapsed += t.elapsed();

            t = Instant::now();
            self.render();
            render_elapsed += t.elapsed();

            t = Instant::now();
            self.process_events();
            events_elapsed += t.elapsed();

            frame_count += 1;

            log_elapsed = last_log.elapsed().as_secs_f64();
            if log_elapsed >= log_delta {
                let frame_time_av = log_elapsed / frame_count as f64;
                let particle_count = self.simulation.particles.len();

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
