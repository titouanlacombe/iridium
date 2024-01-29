use std::time::Instant;

use iridium::examples::benchmark_gravity;
use log::info;

fn main() {
    let t = Instant::now();

    // Start the Tracy client
    tracy_client::Client::start();

    let _span = tracy_client::span!("Main");

    // Configure logging
    env_logger::builder()
        .format_timestamp(None)
        .format_level(true)
        .init();

    let mut app = {
        let _span = tracy_client::span!("App startup");

        // Create the app
        // gravity1(500, 500)
        benchmark_gravity()
    };

    info!("App startup took {} ms", t.elapsed().as_millis());

    // Run the app
    {
        let _span = tracy_client::span!("App run");
        app.run();
    }
}
