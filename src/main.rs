use sdl2;

use iridium::particle::Particle;
use iridium::simulation::Simulation;
use iridium::ui::IridiumRenderer;

fn main() {
    println!("Hello, world!");

    // Build renderer
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Iridium", 800, 600)
        .position_centered()
        .opengl() // TODO switch to vulkan
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();

    let mut iridium_window = IridiumRenderer::new(canvas);

    // Build simulation
    let particles: Vec<Particle> = Vec::new();
    let simulation = Simulation::new(particles);

    // Render loop
    iridium_window.render_loop(&simulation, &mut event_pump);
}
