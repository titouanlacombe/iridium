use nalgebra::Vector2;
use rayon::prelude::*;
use sfml::window::{Event as SfmlEvent, Key};
use std::{
    f64::consts::PI,
    rc::Rc,
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::{
    areas::{Disk, Point, Rect},
    camera,
    forces::{UniformDrag, UniformGravity},
    generators::{
        ConstantGenerator, DiskGenerator, HSVAGenerator, PointGenerator, RGBAGenerator,
        RectGenerator, UniformGenerator, Vector2PolarGenerator,
    },
    integrator::GaussianIntegrator,
    iridium::{max_fps, IridiumMain, SimData},
    particles::{GeneratorFactory, ParticleFactory, Particles},
    random::RngGenerator,
    render_thread::RenderThread,
    renderer::BasicRenderer,
    sim_events::{DefaultSimEventsHandler, SimEvent, SortedVec},
    simulation::{ContinuousSimulationRunner, Simulation, SimulationRunner},
    systems::{ConstantConsumer, ConstantEmitter, Physics, System, VelocityIntegrator, Wall},
    types::Scalar,
    user_events::{BasicUserEventHandler, UserEvent, UserEventCallback},
    window::WindowData,
};

// Basically a facade before i implement the real one
pub fn base_iridium_app(
    width: u32,
    height: u32,
    sim: Simulation,
    sim_runner: Box<dyn SimulationRunner>,
    sim_name: &str,
    min_frame_time: Option<Duration>,
    event_callback: UserEventCallback,
) -> IridiumMain {
    let vertex_buffer = Arc::new(RwLock::new(Vec::new()));

    let camera = Rc::new(RwLock::new(camera::BasicCamera::new(
        Vector2::new(width, height),
        Vector2::zeros(),
        // Vector2::new(-(width as Scalar) / 2., -(height as Scalar) / 2.),
        1.,
        0.,
    )));

    let render_thread = Rc::new(RenderThread::start(
        WindowData::new(
            (width, height),
            format!("Iridium - {}", sim_name),
            sfml::window::Style::CLOSE,
            sfml::window::ContextSettings::default(),
        ),
        vertex_buffer.clone(),
        camera.clone(),
    ));

    IridiumMain::new(
        sim,
        Box::new(BasicRenderer::new(
            render_thread.clone(),
            vertex_buffer,
            min_frame_time,
        )),
        camera,
        sim_runner,
        Box::new(BasicUserEventHandler::new(render_thread, event_callback)),
        4,
        Duration::from_secs(1),
    )
}

fn default_event_handler(data: &mut SimData, event: &UserEvent) {
    let mut camera = data.camera.write().unwrap();

    match event {
        UserEvent::Event(event) => match event {
            SfmlEvent::Closed => {
                data.stop = true;
            }
            SfmlEvent::KeyPressed { code, .. } => match code {
                Key::Escape => {
                    data.stop = true;
                }
                Key::Up | Key::Down | Key::Left | Key::Right => {
                    let zoom = *camera.zoom();
                    match code {
                        Key::Up => camera.offset().y -= 20. / zoom,
                        Key::Down => camera.offset().y += 20. / zoom,
                        Key::Left => camera.offset().x += 20. / zoom,
                        Key::Right => camera.offset().x -= 20. / zoom,
                        _ => {}
                    }
                }
                Key::LShift | Key::LControl => {
                    let rotation_change = match code {
                        Key::LShift => PI * 0.04,
                        Key::LControl => -PI * 0.04,
                        _ => 0.0,
                    };
                    *camera.rotation() += rotation_change;
                }
                Key::Space => {
                    data.running = !data.running;
                }
                _ => {}
            },
            _ => {}
        },
        UserEvent::MouseWheelScrolled { delta, .. } => {
            *camera.zoom() *= 1. + *delta as f64 * 0.05;
        }
        _ => {}
    }
}

pub fn benchmark1() -> IridiumMain {
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
        Box::new(default_event_handler),
    )
}

pub fn benchmark2() -> IridiumMain {
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
        Box::new(default_event_handler),
    )
}

pub fn fireworks(width: u32, height: u32) -> IridiumMain {
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

    let event_callback = Box::new(move |data: &mut SimData, event: &UserEvent| match event {
        UserEvent::MouseButtonPressed {
            button: sfml::window::mouse::Button::Left,
            position,
            ..
        } => {
            let mut pfactory = GeneratorFactory::new(
                Box::new(PointGenerator::new(Point {
                    position: *position,
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
        _ => default_event_handler(data, &event),
    });

    base_iridium_app(
        width,
        height,
        sim,
        sim_runner,
        "Fireworks 2",
        max_fps(60),
        event_callback,
    )
}

pub fn flow(width: u32, height: u32) -> IridiumMain {
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
        Box::new(default_event_handler),
    )
}

struct SimReset;

impl System for SimReset {
    fn update(&mut self, particles: &mut Particles, _dt: Scalar) {
        particles.clear();
    }
}

pub fn benchmark3() -> IridiumMain {
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
        Box::new(default_event_handler),
    )
}

// pub fn events() {}
