use log::debug;
use nalgebra::Vector2;
use sfml::graphics::{Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Vertex};
use sfml::system::Vector2f;
use sfml::window::Event;
use std::time::{Duration, Instant};

use crate::particles::Particles;
use crate::types::{Position, Scalar};

pub trait Renderer {
    fn sim2screen(&self, position: Position) -> Vector2f;
    fn screen2sim(&self, position: Vector2f) -> Position;
    fn render(&mut self, particles: &Particles);
    fn events(&mut self) -> Vec<Event>;
}

pub struct BasicRenderer {
    window: RenderWindow,
    min_frame_time: Option<Duration>,

    // Cache
    screen_size: Vector2<u32>,
    vertex_buffer: Vec<Vertex>,
}

impl BasicRenderer {
    pub fn new(window: RenderWindow, min_frame_time: Option<Duration>) -> Self {
        let mut obj = Self {
            window,
            min_frame_time,
            screen_size: Vector2::new(0, 0),
            vertex_buffer: Vec::new(),
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
    fn sim2screen(&self, position: Position) -> Vector2f {
        Vector2f::new(
            position.x as f32,
            self.screen_size.y as f32 - position.y as f32,
        )
    }

    fn screen2sim(&self, position: Vector2f) -> Position {
        Vector2::new(
            position.x as Scalar,
            self.screen_size.y as Scalar - position.y as Scalar,
        )
    }

    fn render(&mut self, particles: &Particles) {
        let frame_start = Instant::now();

        // Cache current screen size
        self.update_screen_size();

        // Allocate buffers
        self.vertex_buffer.clear();
        self.vertex_buffer.reserve(particles.positions.len());

        // Update position buffer
        for (position, color) in particles.positions.iter().zip(particles.colors.iter()) {
            self.vertex_buffer.push(Vertex::with_pos_color(
                self.sim2screen(*position),
                Color::rgba(
                    (color.0 * 255.) as u8,
                    (color.1 * 255.) as u8,
                    (color.2 * 255.) as u8,
                    (color.3 * 255.) as u8,
                ),
            ));
        }

        // Clear screen
        self.window.clear(Color::BLACK);

        // Draw buffer
        self.window.draw_primitives(
            &self.vertex_buffer,
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
