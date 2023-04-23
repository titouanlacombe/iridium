use iridium::examples::fireworks;

fn main() {
    // Configure logging
    env_logger::builder()
        .format_timestamp(None)
        .format_level(true)
        .init();

    // Run simulation with renderer loop
    fireworks(500, 500).run();
}
