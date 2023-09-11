use nalgebra::Vector2;
use sfml::graphics::{Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Vertex};
use std::{
    ops::Deref,
    rc::Rc,
    sync::{mpsc, Arc, RwLock},
    thread,
};

use crate::{camera::Camera, input::sfml2user_event, user_events::UserEvent, window::WindowData};

pub type VertexBuffer = Arc<RwLock<Vec<Vertex>>>;
type Command = Box<dyn FnOnce(&mut RenderData) + Send>;

pub struct RenderData {
    pub window: RenderWindow,
    pub vertex_buffer: VertexBuffer,
    pub stop: bool,
}

pub struct RenderThread {
    sender: mpsc::Sender<Command>,
    handle: Option<thread::JoinHandle<()>>,
    camera: Rc<RwLock<dyn Camera>>,
}

impl RenderThread {
    pub fn start(
        window: WindowData,
        vertex_buffer: VertexBuffer,
        camera: Rc<RwLock<dyn Camera>>,
    ) -> Self {
        // Create channel
        let (tx, rx) = mpsc::channel::<Command>();

        // Spawn thread
        let handle = thread::spawn(move || {
            // Create SFML window in this thread
            let window: RenderWindow = window.create_real();

            // Create render data
            let mut data = RenderData {
                window,
                vertex_buffer,
                stop: false,
            };

            // Render thread main loop
            loop {
                // Receive & execute command
                rx.recv().unwrap()(&mut data);

                // Check if thread should stop
                if data.stop {
                    break;
                }
            }
        });

        Self {
            sender: tx,
            handle: Some(handle),
            camera,
        }
    }

    fn command(&self, command: Command) {
        self.sender.send(command).unwrap();
    }

    pub fn draw(&self) -> mpsc::Receiver<()> {
        // Create response channel
        let (tx, rx) = mpsc::channel();

        self.command(Box::new(move |data: &mut RenderData| {
            // Clear screen
            data.window.clear(Color::BLACK);

            // Lock buffer
            let vertices = data.vertex_buffer.read().unwrap();

            // Draw buffer
            data.window.draw_primitives(
                vertices.deref(),
                PrimitiveType::POINTS,
                &RenderStates::default(),
            );

            // Release buffer
            drop(vertices);

            // Display
            data.window.display();

            // Send finished signal
            tx.send(()).unwrap();
        }));

        rx
    }

    pub fn get_screen_size(&self) -> Vector2<u32> {
        let (tx, rx) = mpsc::channel();

        self.command(Box::new(move |data: &mut RenderData| {
            let size = data.window.size();
            tx.send(Vector2::new(size.x, size.y)).unwrap();
        }));

        rx.recv().unwrap()
    }

    pub fn get_events(&self) -> Vec<UserEvent> {
        let (tx, rx) = mpsc::channel();

        self.command(Box::new(move |data: &mut RenderData| {
            let mut events = Vec::new();
            while let Some(event) = data.window.poll_event() {
                events.push(event);
            }
            tx.send(events).unwrap();
        }));

        // Receive events and convert them to user events
        let camera = self.camera.read().unwrap();
        rx.recv()
            .unwrap()
            .into_iter()
            .map(|event| sfml2user_event(&event, camera.deref()))
            .collect()
    }
}

impl Drop for RenderThread {
    fn drop(&mut self) {
        // Send stop command
        self.command(Box::new(|data: &mut RenderData| {
            data.stop = true;
        }));

        // Wait for thread to finish
        self.handle.take().unwrap().join().unwrap();
    }
}
