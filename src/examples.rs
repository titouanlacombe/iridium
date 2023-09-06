use nalgebra::Vector2;
use rayon::prelude::*;
use sfml::{
    system::Vector2f,
    window::{Event as SfmlEvent, Key},
};
use std::{
    f64::consts::PI,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    areas::{Disk, Point, Rect},
    events::{DefaultEventsHandler, SimEvent, SortedVec},
    forces::{UniformDrag, UniformGravity},
    generators::{
        ConstantGenerator, DiskGenerator, HSVAGenerator, PointGenerator, RGBAGenerator,
        RectGenerator, UniformGenerator, Vector2PolarGenerator,
    },
    integrator::GaussianIntegrator,
    iridium::{max_fps, IridiumMain},
    particles::{GeneratorFactory, ParticleFactory, Particles},
    random::RngGenerator,
    render_thread::MockRenderWindow,
    renderer::{BasicRenderer, RenderThreadHandle, Renderer},
    simulation::{ContinuousSimulationRunner, Simulation},
    systems::{ConstantConsumer, ConstantEmitter, Physics, System, VelocityIntegrator, Wall},
    types::Scalar,
    user_events::{BasicUserEventHandler, UserEventCallback, UserEventHandler},
};

// Basically a facade before i implement the real one
pub fn sfml_init(
    width: u32,
    height: u32,
    name: &str,
    min_frame_time: Option<Duration>,
    event_callback: UserEventCallback,
) -> (Box<dyn Renderer>, Box<dyn UserEventHandler>) {
    let vertex_buffer = Arc::new(Mutex::new(Vec::new()));

    let render_thread = Arc::new(Mutex::new(RenderThreadHandle::new(
        MockRenderWindow::new(
            (width, height),
            format!("Iridium - {}", name),
            sfml::window::Style::CLOSE,
            sfml::window::ContextSettings::default(),
        ),
        min_frame_time,
        vertex_buffer.clone(),
    )));

    (
        Box::new(BasicRenderer::new(render_thread.clone(), vertex_buffer)),
        Box::new(BasicUserEventHandler::new(render_thread, event_callback)),
    )
}

fn default_event_handler(
    _renderer: &mut Box<dyn Renderer>,
    _sim: &mut Simulation,
    running: &mut bool,
    &event: &SfmlEvent,
) {
    match event {
        SfmlEvent::Closed => {
            *running = false;
        }
        SfmlEvent::KeyPressed { code, .. } => {
            if code == Key::Escape {
                *running = false;
            }
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
            Disk {
                position: Vector2::new(200., 300.),
                radius: 100.,
            },
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

    let (renderer, user_event_handler) = sfml_init(
        width,
        height,
        "Benchmark 1",
        None,
        Box::new(default_event_handler),
    );

    let main = IridiumMain::new(
        sim,
        renderer,
        sim_runner,
        user_event_handler,
        4,
        Duration::from_secs(1),
    );

    main
}

pub fn benchmark2() -> IridiumMain {
    let width = 500;
    let height = 500;
    let mut rng_gen = RngGenerator::new(0);

    let mut factory = GeneratorFactory::new(
        Box::new(DiskGenerator::new(
            Disk {
                position: Vector2::new(200., 300.),
                radius: 100.,
            },
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
        Box::new(Disk {
            // Outside of the simulation
            position: Vector2::new(width as Scalar * 2., height as Scalar * 2.),
            radius: width as Scalar / 10.,
        }),
        100.,
    ));

    let sim = Simulation::new(
        particles,
        vec![limit_cond, consumer, physics, velocity_integrator],
        None,
    );

    let sim_runner = Box::new(ContinuousSimulationRunner::new(1.));

    let (renderer, user_event_handler) = sfml_init(
        width,
        height,
        "Benchmark 2",
        None,
        Box::new(default_event_handler),
    );

    let main = IridiumMain::new(
        sim,
        renderer,
        sim_runner,
        user_event_handler,
        4,
        Duration::from_secs(1),
    );

    main
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

    let event_handler = Box::new(
        move |m_renderer: &mut Box<dyn Renderer>,
              m_sim: &mut Simulation,
              running: &mut bool,
              &event: &SfmlEvent| match event {
            SfmlEvent::MouseButtonPressed {
                button: sfml::window::mouse::Button::Left,
                x,
                y,
                ..
            } => {
                let mut pfactory = GeneratorFactory::new(
                    Box::new(PointGenerator::new(Point {
                        position: m_renderer.screen2sim(Vector2f::new(x as f32, y as f32)),
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

                pfactory.create(1_000, &mut m_sim.particles);
            }
            _ => default_event_handler(m_renderer, m_sim, running, &event),
        },
    );

    let (renderer, user_event_handler) =
        sfml_init(width, height, "Fireworks", max_fps(60), event_handler);

    let main = IridiumMain::new(
        sim,
        renderer,
        sim_runner,
        user_event_handler,
        4,
        Duration::from_secs(1),
    );

    main
}

pub fn flow(width: u32, height: u32) -> IridiumMain {
    let mut rng_gen = RngGenerator::new(0);

    let emitter = Box::new(ConstantEmitter::new(
        Box::new(GeneratorFactory::new(
            Box::new(DiskGenerator::new(
                Disk {
                    position: Vector2::new(
                        width as Scalar / 10.,
                        height as Scalar - (height as Scalar / 10.),
                    ),
                    radius: width as Scalar / 20.,
                },
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
        Box::new(Disk {
            position: Vector2::new(width as Scalar / 2., height as Scalar / 2.),
            radius: width as Scalar / 10.,
        }),
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

    let events_handler = Box::new(DefaultEventsHandler::new(events, 0.));

    let sim = Simulation::new(Particles::new_empty(), systems, Some(events_handler));

    let sim_runner = Box::new(ContinuousSimulationRunner::new(1.));

    let (renderer, user_event_handler) = sfml_init(
        width,
        height,
        "Flow",
        max_fps(60),
        Box::new(default_event_handler),
    );

    let main = IridiumMain::new(
        sim,
        renderer,
        sim_runner,
        user_event_handler,
        4,
        Duration::from_secs(1),
    );

    main
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

    let (renderer, user_event_handler) = sfml_init(
        width,
        height,
        "Benchmark 3",
        None,
        Box::new(default_event_handler),
    );

    let main = IridiumMain::new(
        sim,
        renderer,
        sim_runner,
        user_event_handler,
        4,
        Duration::from_secs(1),
    );

    main
}

// pub fn events() {}
