use nalgebra::Vector2;
use sdl2;

use iridium::particle::{Consumer, Disk, Emitter, RandomFactory};
use iridium::simulation::{LimitCond, Simulation};
use iridium::ui::IridiumRenderer;

fn main() {
    // Global Params
    let width = 800;
    let height = 600;

    // Build renderer
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Iridium", width, height)
        .position_centered()
        .vulkan()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();

    let mut iridium_window = IridiumRenderer::new(canvas);

    // Build simulation
    let emitter = Emitter::new(
        Box::new(RandomFactory::new(
            Box::new(Disk {
                position: Vector2::new(200., 200.),
                radius: 100.,
            }),
            0.5,
            0.5,
            0.,
            2. * std::f32::consts::PI,
            1.,
            1.,
        )),
        1.,
    );

    let consumer = Consumer::new(
        Box::new(Disk {
            position: Vector2::new(400., 500.),
            radius: 100.,
        }),
        1.,
    );

    let mut simulation = Simulation::new(
        Vec::new(),
        vec![emitter],
        vec![consumer],
        LimitCond::Wall(0., 0., width as f32, height as f32, 0.8),
    );

    // Main loop
    iridium_window.render_loop(&mut simulation, &mut event_pump);

    // End
}
