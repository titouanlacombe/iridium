use log::debug;
use rayon::iter::IndexedParallelIterator;
use rayon::prelude::*;
use sfml::graphics::{Color, Vertex};
use sfml::system::Vector2f;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::camera::Camera;
use crate::particles::Particles;
use crate::render_thread::{RenderThread, VertexBuffer};

pub trait Renderer {
    fn render(&mut self, particles: &Particles, camera: &mut dyn Camera);
}

pub struct BasicRenderer {
    render_thread: Rc<RenderThread>,
    vertex_buffer: VertexBuffer,
    min_frame_time: Option<Duration>,

    // Variables
    last_frame: Option<Instant>,
    draw_result: Option<mpsc::Receiver<()>>,
}

impl BasicRenderer {
    pub fn new(
        render_thread: Rc<RenderThread>,
        vertex_buffer: VertexBuffer,
        min_frame_time: Option<Duration>,
    ) -> Self {
        Self {
            render_thread,
            vertex_buffer,
            min_frame_time,
            last_frame: None,
            draw_result: None,
        }
    }

    // Wait for render thread to finish drawing
    fn wait_for_draw(&mut self) {
        if let Some(draw_result) = self.draw_result.take() {
            draw_result.recv().unwrap();
        }
    }
}

impl Renderer for BasicRenderer {
    fn render(&mut self, particles: &Particles, camera: &mut dyn Camera) {
        // Cache current screen size
        *camera.screen_size() = self.render_thread.get_screen_size();

        // Update camera matrices before rendering
        camera.update_matrices();

        // Lock & reserve buffer & coord system
        let mut buffer = self.vertex_buffer.write().unwrap();
        buffer.resize(particles.positions.len(), Vertex::default());

        // Build vertex buffer (par iter on positions and colors)
        particles
            .positions
            .par_iter()
            .zip(particles.colors.par_iter())
            .zip(buffer.par_iter_mut())
            .for_each(|((position, color), vertex)| {
                let pos = camera.sim2screen(*position);
                vertex.position = Vector2f::new(pos.x as f32, pos.y as f32);
                vertex.color = Color::rgba(
                    (color.0 * 255.) as u8,
                    (color.1 * 255.) as u8,
                    (color.2 * 255.) as u8,
                    (color.3 * 255.) as u8,
                );
            });

        // Unlock buffer
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
        self.draw_result = Some(self.render_thread.draw());
    }
}

impl Drop for BasicRenderer {
    fn drop(&mut self) {
        // Wait for last draw to finish for graceful shutdown
        self.wait_for_draw();
    }
}
