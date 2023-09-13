use nalgebra::Vector2;
use rayon::prelude::*;
use sfml::{
    system::Vector2f,
    window::{Event as SfmlEvent, Key},
};
use std::{f64::consts::PI, time::Duration};

use crate::{
    app::{max_fps, AppData, AppMain},
    rendering::{
        input::{KeysState, WindowEvent},
        render_thread::RenderThread,
        renderer::{BasicRenderer, InputCallback, RenderData},
        safe_sfml::{ViewData, WindowData},
    },
    simulation::{
        areas::{Disk, Point, Rect},
        forces::{UniformDrag, UniformGravity},
        generators::{
            ConstantGenerator, DiskGenerator, HSVAGenerator, PointGenerator, RGBAGenerator,
            RectGenerator, UniformGenerator, Vector2PolarGenerator,
        },
        integrator::GaussianIntegrator,
        particles::{GeneratorFactory, ParticleFactory, Particles},
        random::RngGenerator,
        sim_events::{DefaultSimEventsHandler, SimEvent, SortedVec},
        simulation::{ContinuousSimulationRunner, Simulation, SimulationRunner},
        systems::{ConstantConsumer, ConstantEmitter, Physics, System, VelocityIntegrator, Wall},
        types::Scalar,
    },
};

fn get_default_input_callback() -> InputCallback {
    let mut keys_state = KeysState::new();

    Box::new(move |data, render_data, dt, events| {
        let view_data = &mut render_data.view_data.write().unwrap();

        for event in events {
            keys_state.update(&event);

            match event.original {
                SfmlEvent::Closed => {
                    data.stop = true;
                }
                SfmlEvent::KeyPressed { code, .. } => match code {
                    Key::Escape => {
                        data.stop = true;
                    }
                    Key::Space => {
                        data.running = !data.running;
                    }
                    _ => {}
                },
                SfmlEvent::MouseWheelScrolled { delta, .. } => {
                    view_data.zoom *= 1. + delta * 0.05;
                }
                SfmlEvent::Resized { width, height } => {
                    view_data.size = Vector2f::new(width as f32, height as f32);
                }
                _ => {}
            }
        }

        let translation_speed = (dt * 200. / view_data.zoom as f64) as f32;
        let rotation_speed = (dt * 90.) as f32;

        if keys_state.get(Key::Up) {
            view_data.center.y -= translation_speed;
        }
        if keys_state.get(Key::Down) {
            view_data.center.y += translation_speed;
        }
        if keys_state.get(Key::Left) {
            view_data.center.x -= translation_speed;
        }
        if keys_state.get(Key::Right) {
            view_data.center.x += translation_speed;
        }
        if keys_state.get(Key::LShift) {
            view_data.rotation += rotation_speed;
        }
        if keys_state.get(Key::LControl) {
            view_data.rotation -= rotation_speed;
        }
    })
}

// Basically a facade before i implement the real one
pub fn base_iridium_app(
    width: u32,
    height: u32,
    sim: Simulation,
    sim_runner: Box<dyn SimulationRunner>,
    sim_name: &str,
    min_frame_time: Option<Duration>,
    input_callback: InputCallback,
) -> AppMain {
    let view_data = ViewData::new(
        Vector2f::new(width as f32 / 2., height as f32 / 2.),
        Vector2f::new(width as f32, height as f32),
        sfml::graphics::FloatRect::new(0., 0., 1., 1.),
        0.,
        1.,
    );

    let render_data = RenderData::new(view_data);

    let render_thread = RenderThread::start(WindowData::new(
        (width, height),
        format!("Iridium - {}", sim_name),
        sfml::window::Style::DEFAULT,
        sfml::window::ContextSettings::default(),
        false,
    ));

    AppMain::new(
        sim,
        Box::new(BasicRenderer::new(
            render_thread,
            input_callback,
            min_frame_time,
            render_data,
        )),
        sim_runner,
        4,
        Duration::from_secs(1),
    )
}

pub fn benchmark1() -> AppMain {
    let width = 500;
    let height = 500;
    let mut rng_gen = RngGenerator::new(0);

    let mut factory = GeneratorFactory::new(
        Box::new(DiskGenerator::new(
            Disk::new(Vector2::new(200., 300.), 100.),
            rng_gen.next(),
        )),
        Box::new(Vector2PolarGenerator::new(
            Box::new(UniformGenerator::new(rng_gen.next(), 1.2, 1.5)),
            Box::new(UniformGenerator::new(rng_gen.next(), -0.2 * PI, 0.)),
        )),
        Box::new(ConstantGenerator::new(1.)),
        Box::new(RGBAGenerator::new(
            Box::new(ConstantGenerator::new(1.)),
            Box::new(ConstantGenerator::new(1.)),
            Box::new(ConstantGenerator::new(1.)),
            Box::new(ConstantGenerator::new(1.)),
        )),
    );

    let limit_cond = Box::new(Wall {
        x_min: 0.,
        y_min: 0.,
        x_max: width as Scalar,
        y_max: height as Scalar,
        restitution: 0.8,
    });

    let velocity_integrator = Box::new(VelocityIntegrator::new(Box::new(GaussianIntegrator)));

    let mut particles = Particles::new_empty();
    factory.create(1_000_000, &mut particles);

    let sim = Simulation::new(particles, vec![limit_cond, velocity_integrator], None);

    let sim_runner = Box::new(ContinuousSimulationRunner::new(1.));

    base_iridium_app(
        width,
        height,
        sim,
        sim_runner,
        "Benchmark 1",
        None,
        get_default_input_callback(),
    )
}

pub fn benchmark2() -> AppMain {
    let width = 500;
    let height = 500;
    let mut rng_gen = RngGenerator::new(0);

    let mut factory = GeneratorFactory::new(
        Box::new(DiskGenerator::new(
            Disk::new(Vector2::new(200., 300.), 100.),
            rng_gen.next(),
        )),
        Box::new(Vector2PolarGenerator::new(
            Box::new(UniformGenerator::new(rng_gen.next(), 1.2, 1.5)),
            Box::new(UniformGenerator::new(rng_gen.next(), -0.2 * PI, 0.)),
        )),
        Box::new(ConstantGenerator::new(1.)),
        Box::new(RGBAGenerator::new(
            Box::new(ConstantGenerator::new(1.)),
            Box::new(ConstantGenerator::new(1.)),
            Box::new(ConstantGenerator::new(1.)),
            Box::new(ConstantGenerator::new(1.)),
        )),
    );

    let limit_cond = Box::new(Wall {
        x_min: 0.,
        y_min: 0.,
        x_max: width as Scalar,
        y_max: height as Scalar,
        restitution: 0.8,
    });

    let mut particles = Particles::new_empty();
    factory.create(1_000_000, &mut particles);

    let gravity = Box::new(UniformGravity::new(Vector2::new(0., -0.001)));
    let drag = Box::new(UniformDrag::new(0.0005, Vector2::new(-4., 0.)));

    let physics = Box::new(Physics::new(
        vec![gravity, drag],
        Box::new(GaussianIntegrator),
    ));

    let velocity_integrator = Box::new(VelocityIntegrator::new(Box::new(GaussianIntegrator)));

    let consumer = Box::new(ConstantConsumer::new(
        Box::new(Disk::new(
            // Outside of the simulation
            Vector2::new(width as Scalar * 2., height as Scalar * 2.),
            width as Scalar / 10.,
        )),
        100.,
    ));

    let sim = Simulation::new(
        particles,
        vec![limit_cond, consumer, physics, velocity_integrator],
        None,
    );

    let sim_runner = Box::new(ContinuousSimulationRunner::new(1.));

    base_iridium_app(
        width,
        height,
        sim,
        sim_runner,
        "Benchmark 2",
        None,
        get_default_input_callback(),
    )
}

pub fn fireworks(width: u32, height: u32) -> AppMain {
    let mut rng_gen = RngGenerator::new(0);

    let limit_cond = Box::new(Wall {
        x_min: 0.,
        y_min: 0.,
        x_max: width as Scalar,
        y_max: height as Scalar,
        restitution: 0.8,
    });

    let gravity = Box::new(UniformGravity::new(Vector2::new(0., -0.001)));
    let drag = Box::new(UniformDrag::new(0.0005, Vector2::new(-4., 0.)));

    let physics = Box::new(Physics::new(
        vec![gravity, drag],
        Box::new(GaussianIntegrator),
    ));

    let velocity_integrator = Box::new(VelocityIntegrator::new(Box::new(GaussianIntegrator)));

    let sim = Simulation::new(
        Particles::new_empty(),
        vec![limit_cond, physics, velocity_integrator],
        None,
    );

    let sim_runner = Box::new(ContinuousSimulationRunner::new(1.));

    let mut default_input_callback = get_default_input_callback();
    let input_callback = Box::new(
        move |data: &mut AppData,
              render_data: &mut RenderData,
              dt: Scalar,
              events: &Vec<WindowEvent>| {
            default_input_callback(data, render_data, dt, events);

            for event in events {
                match event.original {
                    SfmlEvent::MouseButtonPressed {
                        button: sfml::window::mouse::Button::Left,
                        ..
                    } => {
                        let mut pfactory = GeneratorFactory::new(
                            Box::new(PointGenerator::new(Point {
                                position: event.position.unwrap(),
                            })),
                            Box::new(Vector2PolarGenerator::new(
                                Box::new(UniformGenerator::new(rng_gen.next(), 0., 1.)),
                                Box::new(UniformGenerator::new(rng_gen.next(), 0., 2. * PI)),
                            )),
                            Box::new(ConstantGenerator::new(1.)),
                            Box::new(HSVAGenerator::new(
                                Box::new(UniformGenerator::new(rng_gen.next(), 0., 360.)),
                                Box::new(ConstantGenerator::new(1.)),
                                Box::new(ConstantGenerator::new(1.)),
                                Box::new(ConstantGenerator::new(1.)),
                            )),
                        );

                        pfactory.create(1_000, &mut data.sim.particles);
                    }
                    _ => {}
                }
            }
        },
    );

    base_iridium_app(
        width,
        height,
        sim,
        sim_runner,
        "Fireworks 2",
        max_fps(60),
        input_callback,
    )
}

pub fn flow(width: u32, height: u32) -> AppMain {
    let mut rng_gen = RngGenerator::new(0);

    let emitter = Box::new(ConstantEmitter::new(
        Box::new(GeneratorFactory::new(
            Box::new(DiskGenerator::new(
                Disk::new(
                    Vector2::new(
                        width as Scalar / 10.,
                        height as Scalar - (height as Scalar / 10.),
                    ),
                    width as Scalar / 20.,
                ),
                rng_gen.next(),
            )),
            Box::new(Vector2PolarGenerator::new(
                Box::new(ConstantGenerator::new(0.5)),
                Box::new(ConstantGenerator::new(0.1 * PI)),
            )),
            Box::new(ConstantGenerator::new(1.)),
            Box::new(RGBAGenerator::new(
                Box::new(ConstantGenerator::new(1.)),
                Box::new(ConstantGenerator::new(1.)),
                Box::new(ConstantGenerator::new(1.)),
                Box::new(ConstantGenerator::new(1.)),
            )),
        )),
        30.,
    ));

    let consumer = Box::new(ConstantConsumer::new(
        Box::new(Disk::new(
            Vector2::new(width as Scalar / 2., height as Scalar / 2.),
            width as Scalar / 10.,
        )),
        35.,
    ));

    let limit_cond = Box::new(Wall {
        x_min: 0.,
        y_min: 0.,
        x_max: width as Scalar,
        y_max: height as Scalar,
        restitution: 0.8,
    });

    let gravity = Box::new(UniformGravity::new(Vector2::new(0., -0.001)));
    let drag = Box::new(UniformDrag::new(0.0005, Vector2::new(-0.4, 0.)));

    let physics = Box::new(Physics::new(
        vec![gravity, drag],
        Box::new(GaussianIntegrator),
    ));

    let velocity_integrator = Box::new(VelocityIntegrator::new(Box::new(GaussianIntegrator)));

    // TODO system to change particle color in time
    let systems: Vec<Box<dyn System>> =
        vec![emitter, consumer, limit_cond, physics, velocity_integrator];

    let mut events = SortedVec::new();
    events.add(SimEvent::new(
        5000.,
        Box::new(|particles, systems| {
            // Slow down particles
            particles
                .velocities
                .par_iter_mut()
                .for_each(|vel| *vel *= 0.5);

            // Remove emitter
            systems.remove(0);
        }),
    ));

    let events_handler = Box::new(DefaultSimEventsHandler::new(events, 0.));

    let sim = Simulation::new(Particles::new_empty(), systems, Some(events_handler));

    let sim_runner = Box::new(ContinuousSimulationRunner::new(4.));

    base_iridium_app(
        width,
        height,
        sim,
        sim_runner,
        "Flow",
        max_fps(60),
        get_default_input_callback(),
    )
}

struct SimReset;

impl System for SimReset {
    fn update(&mut self, particles: &mut Particles, _dt: Scalar) {
        particles.clear();
    }
}

pub fn benchmark3() -> AppMain {
    let mut rng_gen = RngGenerator::new(0);
    let width = 500;
    let height = 500;

    let area = Rect {
        position: Vector2::new(0., 0.),
        size: Vector2::new(width as Scalar, height as Scalar),
    };

    let emitter = Box::new(ConstantEmitter::new(
        Box::new(GeneratorFactory::new(
            Box::new(RectGenerator::new(area, rng_gen.next())),
            Box::new(Vector2PolarGenerator::new(
                Box::new(ConstantGenerator::new(0.5)),
                Box::new(ConstantGenerator::new(0.1 * PI)),
            )),
            Box::new(ConstantGenerator::new(1.)),
            Box::new(RGBAGenerator::new(
                Box::new(ConstantGenerator::new(1.)),
                Box::new(ConstantGenerator::new(1.)),
                Box::new(ConstantGenerator::new(1.)),
                Box::new(ConstantGenerator::new(1.)),
            )),
        )),
        5E5,
    ));

    let sim_reseter = Box::new(SimReset);

    let sim = Simulation::new(Particles::new_empty(), vec![sim_reseter, emitter], None);

    let sim_runner = Box::new(ContinuousSimulationRunner::new(1.));

    base_iridium_app(
        width,
        height,
        sim,
        sim_runner,
        "Benchmark 3",
        None,
        get_default_input_callback(),
    )
}

// pub fn events() {}
