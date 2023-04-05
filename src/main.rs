use std::time::Duration;

use iridium::examples::flow;

fn main() {
    // Configure logging
    env_logger::builder()
        .format_timestamp(None)
        .format_level(true)
        .init();

    // Run simulation with renderer loop
    flow(500, 500).main_loop();
}

// TODO move somewhere
// type WindowEventHandler = Box<dyn FnMut(&mut IridiumRenderer, Event)>;
fn _max_fps(fps: u64) -> Option<Duration> {
    Some(Duration::from_micros(1_000_000 / fps))
}
