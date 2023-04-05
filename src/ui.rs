use log::{debug, info};
use nalgebra::Vector2;
use sfml::graphics::{Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Vertex};
use sfml::system::Vector2f;
use sfml::window::{Event, Key};
use std::time::{Duration, Instant};

use crate::areas::Point;
use crate::particle::{ParticleFactory, RandomFactory};
use crate::simulation::SimulationRunner;

pub struct IridiumRenderer {
    pub window: RenderWindow,
    pub sim_runner: Box<dyn SimulationRunner>,
    pub min_frame_time: Option<Duration>,

    pub screen_size: Vector2<f32>,
    pub pos_buffer: Vec<Vertex>,
}

impl IridiumRenderer {
    pub fn new(
        window: RenderWindow,
        sim_runner: Box<dyn SimulationRunner>,
        min_frame_time: Option<Duration>,
    ) -> Self {
        Self {
            window,
            sim_runner,
            min_frame_time,
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

        let particles = &self.sim_runner.get_simulation().particles;

        // Resize particle buffer
        let vertex = Vertex::new(Vector2f::new(0., 0.), Color::WHITE, Vector2f::new(0., 0.));
        self.pos_buffer.resize(particles.len(), vertex);

        // Update position buffer
        let mut i = 0;
        for particle in particles {
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
                    let particles = &mut self.sim_runner.get_simulation_mut().particles;

                    for _ in 0..1000 {
                        particles.push(pfactory.create());
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
        let mut frame_start;
        let mut timer;
        let mut elapsed;

        while self.window.is_open() {
            // TODO refactor timers (search/make utility class)
            frame_start = Instant::now();
            timer = frame_start.clone();

            self.sim_runner.step();

            elapsed = timer.elapsed();
            sim_elapsed += elapsed;
            timer += elapsed;

            self.render();

            elapsed = timer.elapsed();
            render_elapsed += elapsed;
            timer += elapsed;

            self.process_events();

            elapsed = timer.elapsed();
            events_elapsed += elapsed;
            timer += elapsed;

            frame_count += 1;

            log_elapsed = last_log.elapsed().as_secs_f64();
            if log_elapsed >= log_delta {
                let frame_time_av = log_elapsed / frame_count as f64;
                let particles = &self.sim_runner.get_simulation().particles;
                let particle_count = particles.len();

                info!(
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

            if let Some(min_frame_time) = self.min_frame_time {
                let frame_time = frame_start.elapsed();

                if frame_time < min_frame_time {
                    let sleep_time = min_frame_time - frame_time;
                    debug!(
                        "Frame time too short, sleeping for {:.2} ms",
                        sleep_time.as_secs_f64() * 1000.
                    );
                    std::thread::sleep(sleep_time);
                }
            }
        }
    }
}
