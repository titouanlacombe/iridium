use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use sfml::graphics::{Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Vertex};
use std::{
    sync::{mpsc, Arc, RwLock},
    thread,
};

use super::{
    input::WindowEvent,
    safe_sfml::{ViewData, WindowData},
};

pub type DrawResult = Vec<WindowEvent>;

type RenderCommand = Box<dyn FnOnce(&mut RenderData, &mut RenderWindow, &mut bool) + Send>;

#[derive(Clone)]
pub struct RenderData {
    pub vertex_buffer: Arc<RwLock<Vec<Vertex>>>,
    pub view_data: Arc<RwLock<ViewData>>,
}

impl RenderData {
    pub fn new(vertex_buffer: Arc<RwLock<Vec<Vertex>>>, view_data: Arc<RwLock<ViewData>>) -> Self {
        Self {
            vertex_buffer: vertex_buffer,
            view_data: view_data,
        }
    }
}

pub struct RenderThread {
    sender: mpsc::Sender<RenderCommand>,
    handle: Option<thread::JoinHandle<()>>,
}

impl RenderThread {
    pub fn start(window: WindowData, mut data: RenderData) -> Self {
        // Create channel
        let (tx, rx) = mpsc::channel::<RenderCommand>();

        // Spawn thread
        let handle = thread::spawn(move || {
            // Create SFML window in this thread
            let mut window = window.make();

            // Render thread main loop
            let mut stop = false;
            loop {
                // Receive & execute command
                rx.recv().unwrap()(&mut data, &mut window, &mut stop);

                // Check if thread should stop
                if stop {
                    break;
                }
            }
        });

        Self {
            sender: tx,
            handle: Some(handle),
        }
    }

    fn command(&self, command: RenderCommand) {
        self.sender.send(command).unwrap();
    }

    pub fn draw(&self) -> mpsc::Receiver<DrawResult> {
        // Create response channel
        let (tx, rx) = mpsc::channel();

        self.command(Box::new(move |data, window, _stop| {
            // Clear screen
            window.clear(Color::BLACK);

            // Set view
            window.set_view(&data.view_data.read().unwrap().make());

            // Lock buffer
            let mut vertices = data.vertex_buffer.write().unwrap();

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
                events.push(WindowEvent::from_sfml(&event, &window));
            }

            // Send finished signal
            tx.send(events).unwrap();
        }));

        rx
    }
}

impl Drop for RenderThread {
    fn drop(&mut self) {
        // Send stop command
        self.command(Box::new(|_, _, stop| {
            *stop = true;
        }));

        // Wait for thread to finish
        self.handle.take().unwrap().join().unwrap();
    }
}
