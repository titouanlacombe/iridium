use nalgebra::Vector2;
use sdl2::event::Event;
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

    pub fn render(&mut self, simulation: &Simulation) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        for particle in &simulation.particles {
            self.canvas.set_draw_color(Color::RGB(255, 255, 255));
            self.canvas
                .draw_point((particle.position.x as i32, particle.position.y as i32))
                .unwrap();
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

    pub fn render_loop(&mut self, simulation: &mut Simulation, event_pump: &mut EventPump) {
        while self.running {
            let start = std::time::Instant::now();

            simulation.update();
            self.render(simulation);
            self.process_events(simulation, event_pump);

            let elapsed = start.elapsed();
            let elapsed_ms =
                (elapsed.as_secs() as f64) * 1000.0 + (elapsed.subsec_nanos() as f64) / 1_000_000.0;
            // println!("Frame time: {} ms", elapsed_ms);
        }
    }
}
