use log::debug;
use rayon::iter::IndexedParallelIterator;
use rayon::prelude::*;
use sfml::graphics::{Color, Vertex};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use crate::coordinates::{CoordinateSystem, FlippedCoordinateSystem};
use crate::particles::Particles;
use crate::render_thread::{CommandTrait, Draw, GetScreenSize, RenderThreadHandle};

pub trait Renderer {
    fn render(&mut self, particles: &Particles);
}

pub struct BasicRenderer {
    render_thread: Arc<Mutex<RenderThreadHandle>>,
    coord_system: Arc<Mutex<FlippedCoordinateSystem>>,
    vertex_buffer: Arc<Mutex<Vec<Vertex>>>,
    min_frame_time: Option<Duration>,

    // Variables
    last_frame: Option<Instant>,
    draw_result: Option<mpsc::Receiver<<Draw as CommandTrait>::Response>>,
}

impl BasicRenderer {
    pub fn new(
        render_thread: Arc<Mutex<RenderThreadHandle>>,
        coord_system: Arc<Mutex<FlippedCoordinateSystem>>,
        vertex_buffer: Arc<Mutex<Vec<Vertex>>>,
        min_frame_time: Option<Duration>,
    ) -> Self {
        let obj = Self {
            render_thread,
            coord_system,
            vertex_buffer,
            min_frame_time,
            last_frame: None,
            draw_result: None,
        };
        obj.cache_screen_size();
        obj
    }

    fn cache_screen_size(&self) {
        self.coord_system.lock().unwrap().set_screen_size(
            self.render_thread
                .lock()
                .unwrap()
                .command(GetScreenSize)
                .recv()
                .unwrap(),
        );
    }

    // Wait for render thread to finish drawing
    fn wait_for_draw(&mut self) {
        if let Some(draw_result) = self.draw_result.take() {
            draw_result.recv().unwrap();
        }
    }
}

impl Renderer for BasicRenderer {
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

        self.wait_for_draw();

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
        self.last_frame = Some(Instant::now());

        // Send next draw command to render thread
        self.draw_result = Some(self.render_thread.lock().unwrap().command(Draw));
    }
}

impl Drop for BasicRenderer {
    fn drop(&mut self) {
        // Wait for last draw to finish for graceful shutdown
        self.wait_for_draw();
    }
}
