use iridium::forces::UniformGravity;
use nalgebra::Vector2;

use iridium::particle::{Consumer, Disk, Emitter, RandomFactory};
use iridium::renderer;
use iridium::simulation::{LimitCond, Simulation};
use iridium::ui::IridiumUI;

fn main() {
    // Global Params
    let width = 800;
    let height = 600;

    renderer::main();
    // let mut iridium_window = IridiumUI::new(IridiumRenderer::new());

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
        1.,
    );

    let consumer = Consumer::new(
        Box::new(Disk {
            position: Vector2::new(400., 0.),
            radius: 100.,
        }),
        1.,
    );

    let mut simulation = Simulation::new(
        Vec::new(),
        vec![emitter],
        vec![consumer],
        Some(UniformGravity::new(Vector2::new(0., -0.001))),
        None,
        LimitCond::Wall(0., 0., width as f32, height as f32, 0.8),
    );
}
