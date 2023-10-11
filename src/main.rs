use std::time::Instant;

use iridium::examples::gravity1;
use log::info;

fn main() {
    let t = Instant::now();

    // Configure logging
    env_logger::builder()
        .format_timestamp(None)
        .format_level(true)
        .init();

    // Create the app
    let mut app = gravity1(500, 500);
    // let mut app = benchmark2();

    info!("App startup took {} ms", t.elapsed().as_millis());

    // Run the app
    app.run();
}
