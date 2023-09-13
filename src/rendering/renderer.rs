use log::debug;
use rayon::iter::IndexedParallelIterator;
use rayon::prelude::*;
use sfml::graphics::{Color, Vertex};
use sfml::system::Vector2f;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use super::input::WindowEvent;
use super::render_thread::{DrawResult, RenderData, RenderThread};
use crate::app::AppData;
use crate::utils::timer::Timer;

pub type InputCallback = Box<dyn FnMut(&mut AppData, &mut RenderData, f64, &Vec<WindowEvent>)>;

pub trait Renderer {
    fn render(&mut self, sim_data: &mut AppData);
}

pub struct BasicRenderer {
    render_thread: RenderThread,
    input_callback: InputCallback,
    min_frame_time: Option<Duration>,
    render_data: RenderData,

    // Variables
    timer: Timer,
    draw_result: Option<Receiver<DrawResult>>,
}

impl BasicRenderer {
    pub fn new(
        render_thread: RenderThread,
        input_callback: InputCallback,
        min_frame_time: Option<Duration>,
        render_data: RenderData,
    ) -> Self {
        Self {
            render_thread,
            input_callback,
            min_frame_time,
            render_data,
            timer: Timer::new_now(),
            draw_result: None,
        }
    }

    // Wait for render thread to finish drawing
    fn wait_for_draw(&mut self) -> DrawResult {
        if let Some(draw_result) = self.draw_result.take() {
            return draw_result.recv().unwrap();
        }
        vec![]
    }
}

impl Renderer for BasicRenderer {
    fn render(&mut self, data: &mut AppData) {
        let particles = &data.sim.particles;

        // TODO use double buffering to swap buffers for better performance (no need to wait here)
        // TODO Draw command take ref to vertex buffer as argument
        // Wait for last draw to finish & get events since last frame
        let events = self.wait_for_draw();

        // Lock & reserve buffer & coord system
        let mut buffer = self.render_data.vertex_buffer.write().unwrap();
        buffer.resize(particles.positions.len(), Vertex::default());

        // Build vertex buffer
        particles
            .positions
            .par_iter()
            .zip(particles.colors.par_iter())
            .zip(buffer.par_iter_mut())
            .for_each(|((position, color), vertex)| {
                vertex.position = Vector2f::new(position.x as f32, position.y as f32);
                vertex.color = Color::rgba(
                    (color.0 * 255.) as u8,
                    (color.1 * 255.) as u8,
                    (color.2 * 255.) as u8,
                    (color.3 * 255.) as u8,
                );
            });

        // Unlock buffer
        drop(buffer);

        // Handle frame rate limiting
        let mut frame_time = self.timer.elapsed();
        if self.min_frame_time.is_some() {
            let min_frame_time = self.min_frame_time.unwrap();

            if frame_time < min_frame_time {
                let sleep_time = min_frame_time - frame_time;
                frame_time = min_frame_time;
                debug!(
                    "Frame time too short, sleeping for {:.2} ms",
                    sleep_time.as_secs_f64() * 1000.
                );
                std::thread::sleep(sleep_time);
            }
        }
        // Reset timer after sleep
        self.timer.reset();

        // Send next draw command to render thread
        self.draw_result = Some(self.render_thread.draw());

        // Handle events
        (self.input_callback)(
            data,
            &mut self.render_data,
            frame_time.as_secs_f64(),
            &events,
        );
    }
}

impl Drop for BasicRenderer {
    fn drop(&mut self) {
        // Wait for last draw to finish for graceful shutdown
        self.wait_for_draw();
    }
}
