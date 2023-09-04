use log::debug;
use nalgebra::Vector2;
use sfml::graphics::{Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Vertex};
use sfml::system::Vector2f;
use sfml::window::Event;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
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

    // Variables
    vertex_buffer: Arc<Mutex<Vec<Vertex>>>,
    buffer_ready: Arc<Mutex<bool>>,
    screen_size: Vector2<u32>,
    last_frame: Option<Instant>,
}

impl BasicRenderer {
    pub fn new(window: RenderWindow, min_frame_time: Option<Duration>) -> Self {
        let mut obj = Self {
            window,
            min_frame_time,
            vertex_buffer: Arc::new(Mutex::new(Vec::new())),
            buffer_ready: Arc::new(Mutex::new(false)),
            screen_size: Vector2::zeros(),
            last_frame: None,
        };
        obj.cache_screen_size();
        obj
    }

    fn cache_screen_size(&mut self) {
        let tmp = self.window.size();
        self.screen_size.x = tmp.x as u32;
        self.screen_size.y = tmp.y as u32;
    }

    fn draw(&mut self) {
        // Clear screen
        self.window.clear(Color::BLACK);

        // Wait for buffer ready
        loop {
            let mut buffer_ready = self.buffer_ready.lock().unwrap();
            if *buffer_ready {
                *buffer_ready = false;
                drop(buffer_ready);
                break;
            }
        }

        // Lock buffer
        let vertices = self.vertex_buffer.lock().unwrap();

        // Draw buffer
        self.window.draw_primitives(
            vertices.deref(),
            PrimitiveType::POINTS,
            &RenderStates::default(),
        );

        // Release buffer
        drop(vertices);

        // Handle frame rate limiting
        if self.min_frame_time.is_some() && self.last_frame.is_some() {
            let min_frame_time = self.min_frame_time.unwrap();
            let frame_time = self.last_frame.unwrap().elapsed();

            if frame_time < min_frame_time {
                let sleep_time = min_frame_time - frame_time;
                debug!(
                    "Frame time too short, sleeping for {:.2} ms",
                    sleep_time.as_secs_f64() * 1000.
                );
                std::thread::sleep(sleep_time);
            }
        }

        // Display
        self.last_frame = Some(Instant::now());
        self.window.display();
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
        // Cache current screen size
        self.cache_screen_size();

        // Lock & reserve buffer
        let mut buffer = self.vertex_buffer.lock().unwrap();
        buffer.clear();
        buffer.reserve(particles.positions.len());

        // Update position buffer
        for (position, color) in particles.positions.iter().zip(particles.colors.iter()) {
            buffer.push(Vertex::with_pos_color(
                self.sim2screen(*position),
                Color::rgba(
                    (color.0 * 255.) as u8,
                    (color.1 * 255.) as u8,
                    (color.2 * 255.) as u8,
                    (color.3 * 255.) as u8,
                ),
            ));
        }

        // Unlock buffer
        drop(buffer);

        // Wait for not buffer ready
        loop {
            let mut buffer_ready = self.buffer_ready.lock().unwrap();
            if !*buffer_ready {
                *buffer_ready = true;
                drop(buffer_ready);
                break;
            }
        }

        // Draw
        self.draw();
    }

    fn events(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Some(event) = self.window.poll_event() {
            events.push(event);
        }
        events
    }
}
