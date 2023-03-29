use iridium::forces::UniformGravity;
use nalgebra::Vector2;
use sfml::graphics::RenderWindow;

use iridium::particle::{Consumer, Disk, Emitter, RandomFactory};
use iridium::simulation::{LimitCond, Simulation};
use iridium::ui::IridiumRenderer;

fn main() {
    // Global Params
    let width = 800;
    let height = 600;

    // Create window
    let window = RenderWindow::new(
        (width, height),
        "Iridium",
        sfml::window::Style::CLOSE,
        &Default::default(),
    );

    // Create UI
    let mut ui = IridiumRenderer::new(window);

    // Build simulation
    let emitter = Emitter::new(
        Box::new(RandomFactory::new(
            Box::new(Disk {
                position: Vector2::new(200., 400.),
                radius: 100.,
            }),
            0.4,
            0.4,
            0.,
            0.2 * std::f32::consts::PI,
            1.,
            1.,
        )),
        5.,
    );

    let consumer = Consumer::new(
        Box::new(Disk {
            position: Vector2::new(400., 0.),
            radius: 100.,
        }),
        3.,
    );

    let mut simulation = Simulation::new(
        Vec::new(),
        vec![emitter],
        vec![consumer],
        Some(UniformGravity::new(Vector2::new(0., -0.001))),
        None,
        LimitCond::Wall(0., 0., width as f32, height as f32, 0.8),
    );

    ui.render_loop(&mut simulation);
}
