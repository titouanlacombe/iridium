use sfml::graphics::RenderWindow;

use iridium::{examples::flow, ui::IridiumRenderer};
// type WindowEventHandler = Box<dyn FnMut(&mut IridiumRenderer, Event)>;

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
    let mut ui = IridiumRenderer::new(window, flow(width, height));

    // Run simulation with UI loop
    ui.render_loop();
}
