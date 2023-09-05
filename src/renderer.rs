use nalgebra::Vector2;
use rayon::iter::IndexedParallelIterator;
use rayon::prelude::*;
use sfml::graphics::{Color, Vertex};
use sfml::system::Vector2f;
use sfml::window::Event;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use crate::coordinates::{CoordinateSystem, FlippedCoordinateSystem};
use crate::particles::Particles;
use crate::render_thread::commands::{self, Command};
use crate::render_thread::{MockRenderWindow, RenderThread};
use crate::types::Position;

pub trait Renderer {
    fn sim2screen(&self, position: Position) -> Vector2f;
    fn screen2sim(&self, position: Vector2f) -> Position;
    fn render(&mut self, particles: &Particles);
}

pub trait UserEventHandler {
    fn get_events(&self) -> Vec<Event>;
}

pub struct BasicRenderer {
    render_thread: Option<std::thread::JoinHandle<()>>,
    render_thread_channel: mpsc::Sender<Command>,
    draw_result: Option<mpsc::Receiver<()>>,
    vertex_buffer: Arc<Mutex<Vec<Vertex>>>,
    coord_system: Arc<Mutex<FlippedCoordinateSystem>>,
}

impl BasicRenderer {
    pub fn new(window: MockRenderWindow, min_frame_time: Option<Duration>) -> Self {
        let (tx, rx) = mpsc::channel();
        let vertex_buffer = Arc::new(Mutex::new(Vec::new()));
        let coord_system = Arc::new(Mutex::new(FlippedCoordinateSystem::new(Vector2::zeros())));

        let obj = Self {
            render_thread: Some(RenderThread::start(
                window,
                min_frame_time,
                vertex_buffer.clone(),
                rx,
            )),
            render_thread_channel: tx,
            draw_result: None,
            vertex_buffer: vertex_buffer,
            coord_system,
        };
        obj.cache_screen_size();
        obj
    }

    fn cache_screen_size(&self) {
        self.coord_system.lock().unwrap().set_screen_size(
            commands::GetScreenSize
                .send(&self.render_thread_channel)
                .recv()
                .unwrap(),
        );
    }
}

impl Drop for BasicRenderer {
    fn drop(&mut self) {
        commands::Stop
            .send(&self.render_thread_channel)
            .recv()
            .unwrap();
        self.render_thread.take().unwrap().join().unwrap();
    }
}

impl Renderer for BasicRenderer {
    fn sim2screen(&self, position: Position) -> Vector2f {
        self.coord_system.lock().unwrap().sim2screen(position)
    }

    fn screen2sim(&self, position: Vector2f) -> Position {
        self.coord_system.lock().unwrap().screen2sim(position)
    }

    fn render(&mut self, particles: &Particles) {
        // Cache current screen size
        self.cache_screen_size();

        // Lock & reserve buffer & coord system
        let mut buffer = self.vertex_buffer.lock().unwrap();
        buffer.resize(particles.positions.len(), Vertex::default());
        let coord_system = self.coord_system.lock().unwrap();

        // Build vertex buffer (par iter on positions and colors)
        particles
            .positions
            .par_iter()
            .zip(particles.colors.par_iter())
            .zip(buffer.par_iter_mut())
            .for_each(|((position, color), vertex)| {
                vertex.position = coord_system.sim2screen(*position);
                vertex.color = Color::rgba(
                    (color.0 * 255.) as u8,
                    (color.1 * 255.) as u8,
                    (color.2 * 255.) as u8,
                    (color.3 * 255.) as u8,
                );
            });

        // Unlock buffer
        drop(coord_system);
        drop(buffer);

        // Wait end of previous draw
        if let Some(draw_result) = self.draw_result.take() {
            draw_result.recv().unwrap();
        }

        // Send next draw command to render thread
        self.draw_result = Some(commands::Draw.send(&self.render_thread_channel));
    }
}

impl UserEventHandler for BasicRenderer {
    fn get_events(&self) -> Vec<Event> {
        commands::GetEvents
            .send(&self.render_thread_channel)
            .recv()
            .unwrap()
    }
}
