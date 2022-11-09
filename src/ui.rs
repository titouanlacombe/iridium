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
        IridiumRenderer {
            canvas,
            running: true,
        }
    }

    pub fn render_particle(&mut self, particle: &Particle) {
        self.canvas
            .filled_circle(
                particle.position.x as i16,
                particle.position.y as i16,
                5,
                Color::RGB(255, 255, 255),
            )
            .unwrap();
    }

    pub fn render(&mut self, simulation: &Simulation) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        for particle in &simulation.particles {
            self.render_particle(particle);
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
                    simulation.add_particle(Particle::new(
                        Vector2::new(x as f32, y as f32),
                        Vector2::new(1., 1.),
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
        let mut start = std::time::Instant::now();
        let mut frame_count = 0;

        while self.running {
            simulation.update();
            self.render(simulation);
            self.process_events(simulation, event_pump);

            frame_count += 1;

            if last_log.elapsed().as_secs_f32() >= 1. {
                let av_frame_time = start.elapsed().as_secs_f32() / frame_count as f32;

                println!(
                    "~{} ms ({} fps)",
                    av_frame_time * 1000.,
                    (1. / av_frame_time) as i32
                );

                last_log = std::time::Instant::now();
                start = std::time::Instant::now();
                frame_count = 0;
            }
        }
    }
}
