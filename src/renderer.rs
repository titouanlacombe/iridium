use rayon::iter::IndexedParallelIterator;
use rayon::prelude::*;
use sfml::graphics::{Color, Vertex};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use crate::coordinates::{CoordinateSystem, FlippedCoordinateSystem};
use crate::particles::Particles;
use crate::render_thread::{
    CommandEnum, Draw, GetScreenSize, MockRenderWindow, RenderThread, Stop,
};

pub trait Renderer {
    fn render(&mut self, particles: &Particles);
}

pub struct RenderThreadHandle {
    pub channel: mpsc::Sender<CommandEnum>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl RenderThreadHandle {
    pub fn new(
        window: MockRenderWindow,
        min_frame_time: Option<Duration>,
        vertex_buffer: Arc<Mutex<Vec<Vertex>>>,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            channel: tx,
            handle: Some(RenderThread::start(
                window,
                min_frame_time,
                vertex_buffer,
                rx,
            )),
        }
    }

    // TODO fix this
    // pub fn command<T: CommandEnum>(&self, command: T) -> mpsc::Receiver<T::Response> {
    //     let (tx, rx) = mpsc::channel();
    //     self.channel.send(command).unwrap();
    //     rx
    // }
}

impl Drop for RenderThreadHandle {
    fn drop(&mut self) {
        Stop.send(&self.channel).recv().unwrap();
        self.handle.take().unwrap().join().unwrap();
    }
}

pub struct BasicRenderer {
    render_thread: Arc<Mutex<RenderThreadHandle>>,
    vertex_buffer: Arc<Mutex<Vec<Vertex>>>,
    coord_system: Arc<Mutex<FlippedCoordinateSystem>>,
    draw_result: Option<mpsc::Receiver<()>>,
}

impl BasicRenderer {
    pub fn new(
        render_thread: Arc<Mutex<RenderThreadHandle>>,
        vertex_buffer: Arc<Mutex<Vec<Vertex>>>,
        coord_system: Arc<Mutex<FlippedCoordinateSystem>>,
    ) -> Self {
        let obj = Self {
            render_thread,
            vertex_buffer,
            coord_system,
            draw_result: None,
        };
        obj.cache_screen_size();
        obj
    }

    fn cache_screen_size(&self) {
        self.coord_system.lock().unwrap().set_screen_size(
            GetScreenSize
                .send(&self.render_thread.lock().unwrap().channel)
                .recv()
                .unwrap(),
        );
    }

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

        // Send next draw command to render thread
        self.draw_result = Some(Draw.send(&self.render_thread.lock().unwrap().channel));
    }
}

impl Drop for BasicRenderer {
    fn drop(&mut self) {
        self.wait_for_draw();
    }
}
