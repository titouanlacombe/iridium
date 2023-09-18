use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use sfml::graphics::{Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Vertex};
use std::sync::{
    mpsc::{self, Receiver},
    Arc, RwLock,
};

use super::{
    input::WindowEvent,
    safe_sfml::{ViewData, WindowData},
};
use crate::utils::worker_thread::WorkerThread;

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

impl RenderThread {
    pub fn new(window: WindowData) -> Self {
        let thread = WorkerThread::new();

        // Create window
        thread.send(Box::new(move |data: &mut RenderThreadData, _stop| {
            data.window = Some(window.make());
        }));

        Self { thread }
    }

    pub fn draw(
        &self,
        buffer: Arc<RwLock<Vec<Vertex>>>,
        view_data: Arc<RwLock<ViewData>>,
    ) -> Receiver<DrawResult> {
        // Create response channel
        let (tx, rx) = mpsc::channel();

        self.thread.send(Box::new(move |data, _stop| {
            let window = data.window.as_mut().unwrap();

            // Clear screen
            window.clear(Color::BLACK);

            // Set view
            window.set_view(&view_data.read().unwrap().make());

            // Lock buffer
            let mut vertices = buffer.write().unwrap();

            // Flip y axis
            let size_y = window.size().y as f32;
            vertices.par_iter_mut().for_each(|v| {
                v.position.y = size_y - v.position.y;
            });

            // Draw buffer
            window.draw_primitives(&vertices, PrimitiveType::POINTS, &RenderStates::default());

            // Release buffer
            drop(vertices);

            // Display
            window.display();

            // Poll events
            let mut events = Vec::new();
            while let Some(event) = window.poll_event() {
                events.push(WindowEvent::from_sfml(event, &window));
            }

            // Send finished signal
            tx.send(events).unwrap();
        }));

        rx
    }
}
