use nalgebra::Vector2;
use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::EventPump;

use crate::particle::Particle;
use crate::simulation::Simulation;

pub struct IridiumRenderer<T: RenderTarget> {
    pub canvas: Canvas<T>,
    pub running: bool,
}

impl<T: RenderTarget> IridiumRenderer<T> {
    pub fn new(canvas: Canvas<T>) -> IridiumRenderer<T> {
        Self {
            canvas,
            running: true,
        }
    }

    // Convert simulation position to screen position
    pub fn sim2screen(&self, sim: &Simulation, position: Vector2<f32>) -> (i16, i16) {
        (
            position.x as i16,
            self.canvas.viewport().height() as i16 - position.y as i16,
        )
    }

    pub fn render_particle(&mut self, simulation: &Simulation, particle: &Particle) {
        let (x, y) = self.sim2screen(&simulation, particle.position);
        self.canvas
            .filled_circle(x, y, 2, Color::RGB(255, 255, 255))
            .unwrap();
    }

    pub fn render(&mut self, simulation: &Simulation) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        for particle in &simulation.particles {
            self.render_particle(simulation, particle);
        }

        self.canvas.present();
    }

    pub fn process_events(&mut self, simulation: &mut Simulation, event_pump: &mut EventPump) {
        for event in event_pump.poll_iter() {
            match event {
                // Exit on escape or Quit event
                Event::Quit { .. } => self.running = false,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => self.running = false,

                // Spawn a particle on mouse click
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
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
    pub fn render_loop(&mut self, simulation: &mut Simulation, event_pump: &mut EventPump) {
        let mut last_log = std::time::Instant::now();
        let mut frame_count = 0;
        let mut elapsed: f32;
        let log_delta = 1.0; // Log every second

        while self.running {
            simulation.update();
            self.render(simulation);
            self.process_events(simulation, event_pump);

            frame_count += 1;

            elapsed = last_log.elapsed().as_secs_f32();
            if elapsed >= log_delta {
                let av_frame_time = elapsed / frame_count as f32;

                println!(
                    "{} frames in {} s\n~{} ms/frame ({} fps)\n{} particles ({} µs/particle)",
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
