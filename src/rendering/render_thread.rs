use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use sfml::{
    graphics::{Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Vertex},
    system::Vector2f,
};
use std::sync::{
    mpsc::{self, Receiver},
    Arc, RwLock,
};

use super::{
    input::WindowEvent,
    safe_sfml::{ViewData, WindowData},
};
use crate::{simulation::areas::Rect, utils::worker_thread::WorkerThread};

pub type DrawResult = Vec<WindowEvent>;

pub struct RenderThreadData {
    pub window: Option<RenderWindow>,
}

impl Default for RenderThreadData {
    fn default() -> Self {
        Self { window: None }
    }
}

pub struct RenderThread {
    thread: WorkerThread<RenderThreadData>,
}

pub fn sfml_to_nalgebra(v: Vector2f) -> nalgebra::Vector2<f32> {
    nalgebra::Vector2::new(v.x, v.y)
}

pub fn nalgebra32_to_sfml(v: nalgebra::Vector2<f32>) -> Vector2f {
    Vector2f::new(v.x, v.y)
}

pub fn nalgebra64_to_sfml(v: nalgebra::Vector2<f64>) -> Vector2f {
    Vector2f::new(v.x as f32, v.y as f32)
}

impl RenderThread {
    pub fn new(window: WindowData) -> Self {
        let thread = WorkerThread::new();

        // Initialize thread
        thread.send(Box::new(move |data: &mut RenderThreadData, _stop| {
            // Create window
            data.window = Some(window.make());

            // Thread name
            tracy_client::set_thread_name!("Render Thread");
        }));

        Self { thread }
    }

    fn flip_y(window: &mut RenderWindow, vertices: &mut Vec<Vertex>, use_parallel: bool) {
        let size_y = window.size().y as f32;
        let lambda = |v: &mut Vertex| {
            v.position.y = size_y - v.position.y;
        };

        if use_parallel {
            vertices.par_iter_mut().for_each(lambda);
        } else {
            vertices.iter_mut().for_each(lambda);
        }
    }

    pub fn draw(
        &self,
        buffer: Arc<RwLock<Vec<Vertex>>>,
        quadtree_primitives: Arc<RwLock<Vec<(Rect, Color)>>>,
        view_data: Arc<RwLock<ViewData>>,
    ) -> Receiver<DrawResult> {
        // Create response channel
        let (tx, rx) = mpsc::channel();

        self.thread.send(Box::new(move |data, _stop| {
            let _span = tracy_client::span!("Render Thread");

            let window = {
                let _span = tracy_client::span!("Init window");

                let window = data.window.as_mut().unwrap();

                // Clear screen
                window.clear(Color::BLACK);

                // Set view
                window.set_view(&view_data.read().unwrap().make());

                window
            };

            // Draw QuadTree
            {
                let _span = tracy_client::span!("Draw QuadTree");

                let quadtree_primitives = quadtree_primitives.read().unwrap();
                for (rect, color) in quadtree_primitives.iter() {
                    let positions = [
                        rect.top_left(),
                        rect.top_right(),
                        rect.bottom_right(),
                        rect.bottom_left(),
                        rect.top_left(),
                    ];

                    let mut vertices = positions
                        .iter()
                        .map(|p| Vertex::with_pos_color(nalgebra64_to_sfml(*p), *color))
                        .collect::<Vec<_>>();

                    // Flip y axis
                    Self::flip_y(window, &mut vertices, false);

                    window.draw_primitives(
                        vertices.as_slice(),
                        PrimitiveType::LINE_STRIP,
                        &RenderStates::default(),
                    );
                }
            }

            // Lock buffer
            let mut vertices = {
                let _span = tracy_client::span!("Lock buffer");
                buffer.write().unwrap()
            };

            // Flip y axis
            {
                let _span = tracy_client::span!("Flip y axis");
                Self::flip_y(window, &mut vertices, true);
            }

            // Draw buffer
            {
                let _span = tracy_client::span!("Draw buffer");
                window.draw_primitives(&vertices, PrimitiveType::POINTS, &RenderStates::default());
            }

            // Release buffer
            drop(vertices);

            // Display
            {
                let _span = tracy_client::span!("Display");
                window.display();
            }

            // Poll events
            let events = {
                let _span = tracy_client::span!("Poll events");

                let mut events = Vec::new();
                while let Some(event) = window.poll_event() {
                    events.push(WindowEvent::from_sfml(event, &window));
                }
                events
            };

            // Send finished signal
            tx.send(events).unwrap();
        }));

        rx
    }
}
