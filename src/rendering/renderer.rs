use log::debug;
use rayon::iter::IndexedParallelIterator;
use rayon::prelude::*;
use sfml::graphics::{Color as SfmlColor, Vertex};
use sfml::system::Vector2f;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use super::input::WindowEvent;
use super::render_thread::{DrawResult, RenderThread};
use super::safe_sfml::ViewData;
use crate::app::AppData;
use crate::simulation::areas::Rect;
use crate::simulation::quadtree::QuadTree;
use crate::utils::timer::Timer;

pub type InputCallback = Box<dyn FnMut(&mut AppData, &mut RenderData, f64, &Vec<WindowEvent>)>;

pub trait Renderer {
    fn render(&mut self, sim_data: &mut AppData);
}

#[derive(Clone)]
pub struct RenderData {
    pub view_data: Arc<RwLock<ViewData>>,
    pub vertex_buffer_a: Arc<RwLock<Vec<Vertex>>>,
    pub vertex_buffer_b: Arc<RwLock<Vec<Vertex>>>,
    pub quadtree_vertex_buffer: Arc<RwLock<Vec<(Rect, SfmlColor)>>>,
}

impl RenderData {
    pub fn new(view_data: ViewData) -> Self {
        Self {
            view_data: Arc::new(RwLock::new(view_data)),
            vertex_buffer_a: Arc::new(RwLock::new(Vec::new())),
            vertex_buffer_b: Arc::new(RwLock::new(Vec::new())),
            quadtree_vertex_buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

pub struct BasicRenderer {
    quadtree: Option<Arc<RwLock<QuadTree>>>,

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
        quadtree: Option<Arc<RwLock<QuadTree>>>,
        render_thread: RenderThread,
        input_callback: InputCallback,
        min_frame_time: Option<Duration>,
        render_data: RenderData,
    ) -> Self {
        Self {
            quadtree,
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
    // TODO Remove AppData and put dependencies in constructor
    fn render(&mut self, data: &mut AppData) {
        let particles = &data.sim.particles;

        // Lock & reserve buffer & coord system
        let mut buffer = self.render_data.vertex_buffer_a.write().unwrap();
        buffer.resize(particles.positions.len(), Vertex::default());

        // Build vertex buffer
        particles
            .positions
            .par_iter()
            .zip(particles.colors.par_iter())
            .zip(buffer.par_iter_mut())
            .for_each(|((position, color), vertex)| {
                vertex.position = Vector2f::new(position.x as f32, position.y as f32);
                vertex.color = SfmlColor::rgba(
                    (color.r * 255.) as u8,
                    (color.g * 255.) as u8,
                    (color.b * 255.) as u8,
                    (color.a * 255.) as u8,
                );
            });

        // Unlock buffer
        drop(buffer);

        // Quadtree
        if let Some(quadtree) = &self.quadtree {
            let qt = quadtree.read().unwrap();

            // Lock buffer
            let mut buffer = self.render_data.quadtree_vertex_buffer.write().unwrap();
            buffer.clear();

            // Build vertex buffer
            let k = 0.5; // Rate of color change

            let mut stack = vec![(&qt.root, 0)];
            while let Some((node, depth)) = stack.pop() {
                // Get rect params
                let rect = node.rect.clone();

                // Color based on depth (from green to red)
                let capped_depth = 1. - (1. / (k * (depth as f64) + 1.));
                let color = SfmlColor::rgba(
                    (capped_depth * 255.) as u8,
                    ((1. - capped_depth) * 255.) as u8,
                    0,
                    255,
                );

                // Draw rect vertices
                buffer.push((rect, color));

                // Draw children
                for child in &node.childs {
                    stack.push((child, depth + 1));
                }
            }

            // Unlock buffer
            drop(buffer);
        }

        // Swap buffers
        std::mem::swap(
            &mut self.render_data.vertex_buffer_a,
            &mut self.render_data.vertex_buffer_b,
        );

        // Wait for last draw to finish & get events since last frame
        let events = self.wait_for_draw();

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
        self.draw_result = Some(self.render_thread.draw(
            self.render_data.vertex_buffer_b.clone(),
            self.render_data.quadtree_vertex_buffer.clone(),
            self.render_data.view_data.clone(),
        ));

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

pub struct NoRenderer {}

impl NoRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderer for NoRenderer {
    fn render(&mut self, _data: &mut AppData) {}
}
