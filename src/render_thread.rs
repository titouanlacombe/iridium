use std::{
    ops::Deref,
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
};

use log::debug;
use nalgebra::Vector2;
use sfml::{
    graphics::{Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Vertex},
    window::Event as SFMLEvent,
};

macro_rules! DefineCommands {
	($($name:ident($res:ty)),+ $(,)?) => {
		// Define the Command enum
		pub enum CommandEnum {
			$(
				$name(mpsc::Sender<$res>),
			)+
		}

		// Define individual command structs
		$(
			pub struct $name;

			impl $name {
				pub fn send(self, sender: &mpsc::Sender<CommandEnum>) -> mpsc::Receiver<$res> {
					let (tx, rx) = mpsc::channel();
					sender.send(CommandEnum::$name(tx)).unwrap();
					rx
				}
			}
		)+
	};
}

DefineCommands! {
    Draw(()),
    GetScreenSize(Vector2<u32>),
    GetEvents(Vec<SFMLEvent>),
    Stop(()),
}

pub struct MockRenderWindow {
    pub size: (u32, u32),
    pub title: String,
    pub style: sfml::window::Style,
    pub settings: sfml::window::ContextSettings,
}

impl MockRenderWindow {
    pub fn new(
        size: (u32, u32),
        title: String,
        style: sfml::window::Style,
        settings: sfml::window::ContextSettings,
    ) -> Self {
        Self {
            size,
            title,
            style,
            settings,
        }
    }
}

pub struct RenderThread {
    window: RenderWindow,
    min_frame_time: Option<Duration>,

    // Variables
    vertex_buffer: Arc<Mutex<Vec<Vertex>>>,
    last_frame: Option<Instant>,
}

impl RenderThread {
    pub fn new(
        window: RenderWindow,
        vertex_buffer: Arc<Mutex<Vec<Vertex>>>,
        min_frame_time: Option<Duration>,
    ) -> Self {
        Self {
            window,
            vertex_buffer,
            min_frame_time,
            last_frame: None,
        }
    }

    fn draw(&mut self) {
        // Clear screen
        self.window.clear(Color::BLACK);

        // Lock buffer
        let vertices = self.vertex_buffer.lock().unwrap();

        // Draw buffer
        self.window.draw_primitives(
            vertices.deref(),
            PrimitiveType::POINTS,
            &RenderStates::default(),
        );

        // Release buffer
        drop(vertices);

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

        // Display
        self.last_frame = Some(Instant::now());
        self.window.display();
    }

    fn get_screen_size(&mut self) -> Vector2<u32> {
        let tmp = self.window.size();
        Vector2::new(tmp.x as u32, tmp.y as u32)
    }

    fn events(&mut self) -> Vec<SFMLEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.window.poll_event() {
            events.push(event);
        }
        events
    }

    pub fn main_loop(&mut self, rx: mpsc::Receiver<CommandEnum>) {
        loop {
            // Receive and handle command
            let command = rx.recv().unwrap();
            match command {
                CommandEnum::Draw(tx) => {
                    self.draw();
                    tx.send(()).unwrap();
                }
                CommandEnum::GetScreenSize(tx) => {
                    tx.send(self.get_screen_size()).unwrap();
                }
                CommandEnum::GetEvents(tx) => {
                    tx.send(self.events()).unwrap();
                }
                CommandEnum::Stop(tx) => {
                    tx.send(()).unwrap();
                    break;
                }
            }
        }
    }

    pub fn start(
        mock_window: MockRenderWindow,
        min_frame_time: Option<Duration>,
        vertex_buffer: Arc<Mutex<Vec<Vertex>>>,
        rx: mpsc::Receiver<CommandEnum>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            RenderThread::new(
                RenderWindow::new(
                    mock_window.size,
                    mock_window.title.as_str(),
                    mock_window.style,
                    &mock_window.settings,
                ),
                vertex_buffer,
                min_frame_time,
            )
            .main_loop(rx);
        })
    }
}
