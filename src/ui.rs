use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::EventPump;

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
        self.canvas.present();
    }

    pub fn process_events(&mut self, event_pump: &mut EventPump) {
        // Exit on escape or Quit event
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => self.running = false,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => self.running = false,
                _ => {}
            }
        }
    }

    pub fn render_loop(&mut self, simulation: &Simulation, event_pump: &mut EventPump) {
        while self.running {
            self.render(simulation);
            self.process_events(event_pump);
        }
    }
}
