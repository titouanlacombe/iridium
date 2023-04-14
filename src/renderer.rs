use log::debug;
use nalgebra::Vector2;
use sfml::graphics::{Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Vertex};
use sfml::system::Vector2f;
use sfml::window::Event;
use std::time::{Duration, Instant};

use crate::particle::Particles;

pub trait Renderer {
    fn sim2screen(&self, position: Vector2<f32>) -> Vector2f;
    fn screen2sim(&self, position: Vector2f) -> Vector2<f32>;
    fn render(&mut self, particles: &Particles);
    fn events(&mut self) -> Vec<Event>;
}

pub struct BasicRenderer {
    window: RenderWindow,
    min_frame_time: Option<Duration>,

    // Cache
    screen_size: Vector2<u32>,
    pos_buffer: Vec<Vertex>,
}

impl BasicRenderer {
    pub fn new(window: RenderWindow, min_frame_time: Option<Duration>) -> Self {
        let mut obj = Self {
            window,
            min_frame_time,
            screen_size: Vector2::new(0, 0),
            pos_buffer: Vec::new(),
        };
        obj.update_screen_size();
        obj
    }

    fn update_screen_size(&mut self) {
        let tmp = self.window.size();
        self.screen_size.x = tmp.x as u32;
        self.screen_size.y = tmp.y as u32;
    }
}

impl Renderer for BasicRenderer {
    fn sim2screen(&self, position: Vector2<f32>) -> Vector2f {
        Vector2f::new(position.x, self.screen_size.y as f32 - position.y)
    }

    fn screen2sim(&self, position: Vector2f) -> Vector2<f32> {
        Vector2::new(position.x, self.screen_size.y as f32 - position.y)
    }

    fn render(&mut self, particles: &Particles) {
        let frame_start = Instant::now();

        // Cache current screen size
        self.update_screen_size();

        // Resize particle buffer
        let vertex = Vertex::new(Vector2f::new(0., 0.), Color::WHITE, Vector2f::new(0., 0.));
        self.pos_buffer.resize(particles.len(), vertex);

        // Update position buffer
        let mut i = 0;
        for position in particles.positions.iter() {
            self.pos_buffer[i].position = self.sim2screen(*position);
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

        // Handle frame rate limiting
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

    fn events(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Some(event) = self.window.poll_event() {
            events.push(event);
        }
        events
    }
}
