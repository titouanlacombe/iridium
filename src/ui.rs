use nalgebra::Vector2;
use sfml::graphics::{Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Vertex};
use sfml::system::Vector2f;
use sfml::window::{Event, Key};

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
                let particle_count = simulation.particles.len();

                println!(
                    "{} steps in {:.2} s\n\
					{:.2} ms/frame ({:.2} fps)\n\
					{:.0e} particles ({:.2} µs/particle)\n",
                    frame_count,
                    elapsed,
                    frame_time_av * 1000.,
                    (1. / frame_time_av) as i32,
                    particle_count,
                    ((elapsed * 1_000_000.) / frame_count as f32) / particle_count as f32
                );

                last_log = std::time::Instant::now();
                frame_count = 0;
            }
        }
    }
}
