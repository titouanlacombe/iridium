use sfml::graphics::RenderWindow;
use std::time::Duration;

use iridium::{examples::flow, renderer::IridiumRenderer};
// type WindowEventHandler = Box<dyn FnMut(&mut IridiumRenderer, Event)>;

// TODO move somewhere
fn _max_fps(fps: u64) -> Option<Duration> {
    Some(Duration::from_micros(1_000_000 / fps))
}

fn main() {
    // Configure logging
    env_logger::builder()
        .format_timestamp(None)
        .format_level(true)
        .init();

    // Global Params
    let width = 500;
    let height = 500;

    // Create window
    let window = RenderWindow::new(
        (width, height),
        "Iridium",
        sfml::window::Style::CLOSE,
        &Default::default(),
    );

    // Create UI
    let mut renderer =
        IridiumRenderer::new(window, flow(width, height), None, Duration::from_secs(1));

    // Run simulation with renderer loop
    renderer.main_loop();
}
