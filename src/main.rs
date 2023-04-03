use sfml::graphics::RenderWindow;

use iridium::{examples::benchmark1, ui::IridiumRenderer};

fn main() {
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
    let mut ui = IridiumRenderer::new(window, benchmark1());

    // Run simulation with UI loop
    ui.render_loop();
}
