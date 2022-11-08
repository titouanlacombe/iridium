use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::EventPump;

use crate::simulation::Simulation;

pub struct IridiumRenderer<T: RenderTarget> {
    pub canvas: Canvas<T>,
    pub running: bool,
    pub event_pump: EventPump,
}

impl<T: RenderTarget> IridiumRenderer<T> {
    pub fn new(canvas: Canvas<T>, event_pump: EventPump) -> IridiumRenderer<T> {
        IridiumRenderer {
            canvas,
            running: true,
            event_pump,
        }
    }

    pub fn render(simulation: &Simulation) {
        println!("Rendering simulation");
    }

    pub fn process_events(&mut self) {
        // Exit on escape or Quit event
        for event in self.event_pump.poll_iter() {
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

    pub fn render_loop(&mut self) {
        while self.running {
            self.canvas.set_draw_color(Color::RGB(0, 0, 0));
            self.canvas.clear();
            self.canvas.present();

            self.process_events();
        }
    }
}
