use nalgebra::Vector2;
use sfml::graphics::{Color, Vertex};
use sfml::system::Vector2f;
use sfml::window::Event;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use crate::particles::Particles;
use crate::render_thread::commands::{self, Command};
use crate::render_thread::{MockRenderWindow, RenderThread};
use crate::types::{Position, Scalar};

pub trait Renderer {
    fn sim2screen(&self, position: Position) -> Vector2f;
    fn screen2sim(&self, position: Vector2f) -> Position;
    fn render(&mut self, particles: &Particles);
    fn events(&mut self) -> Vec<Event>;
}

pub struct BasicRenderer {
    render_thread: std::thread::JoinHandle<()>,
    render_thread_channel: mpsc::Sender<Command>,
    draw_result: Option<mpsc::Receiver<()>>,
    vertex_buffer: Arc<Mutex<Vec<Vertex>>>,
    screen_size: Vector2<u32>,
}

impl BasicRenderer {
    pub fn new(window: MockRenderWindow, min_frame_time: Option<Duration>) -> Self {
        let (tx, rx) = mpsc::channel();
        let vertex_buffer = Arc::new(Mutex::new(Vec::new()));

        let mut obj = Self {
            render_thread: RenderThread::start(window, min_frame_time, vertex_buffer.clone(), rx),
            render_thread_channel: tx,
            draw_result: None,
            vertex_buffer: vertex_buffer,
            screen_size: Vector2::zeros(),
        };
        obj.cache_screen_size();
        obj
    }

    fn cache_screen_size(&mut self) {
        let tmp = commands::GetScreenSize
            .send(&self.render_thread_channel)
            .recv()
            .unwrap();
        self.screen_size.x = tmp.x;
        self.screen_size.y = tmp.y;
    }

    // TODO add to interface?
    pub fn stop(self) {
        commands::Stop
            .send(&self.render_thread_channel)
            .recv()
            .unwrap();
        self.render_thread.join().unwrap();
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

        // Wait end of previous draw
        if let Some(draw_result) = self.draw_result.take() {
            draw_result.recv().unwrap();
        }

        // Send next draw command to render thread
        self.draw_result = Some(commands::Draw.send(&self.render_thread_channel));
    }

    fn events(&mut self) -> Vec<Event> {
        commands::GetEvents
            .send(&self.render_thread_channel)
            .recv()
            .unwrap()
    }
}
