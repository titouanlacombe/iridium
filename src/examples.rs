use nalgebra::Vector2;
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use sfml::{
    graphics::RenderWindow,
    system::Vector2f,
    window::{Event as SfmlEvent, Key},
};
use std::{f64::consts::PI, time::Duration};

use crate::{
    areas::{Disk, Point, Rect},
    events::{Event, EventsHandler, SortedVec},
    forces::{UniformDrag, UniformGravity},
    generators::{
        ConstantGenerator, DiskGenerator, PointGenerator, RectGenerator, UniformGenerator,
        Vector2PolarGenerator,
    },
    iridium::IridiumMain,
    particle::{GeneratorFactory, ParticleFactory, Particles},
    renderer::{BasicRenderer, Renderer},
    simulation::{ContinuousSimulationRunner, Simulation},
    systems::{ConstantConsumer, ConstantEmitter, GaussianIntegrator, Physics, System, Wall},
    types::Scalar,
};

pub fn get_window(width: u32, height: u32) -> RenderWindow {
    let window = RenderWindow::new(
        (width, height),
        "Iridium",
        sfml::window::Style::CLOSE,
        &Default::default(),
    );

    window
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
    let seed: u64 = 0;

    let mut factory = GeneratorFactory::new(
        Box::new(DiskGenerator::new(
            Disk {
                position: Vector2::new(200., 300.),
                radius: 100.,
            },
            XorShiftRng::seed_from_u64(seed),
        )),
        Box::new(Vector2PolarGenerator::new(
            Box::new(UniformGenerator::new(
                XorShiftRng::seed_from_u64(seed),
                1.2,
                1.5,
            )),
            Box::new(UniformGenerator::new(
                XorShiftRng::seed_from_u64(seed),
                -0.2 * PI,
                0.,
            )),
        )),
        Box::new(ConstantGenerator::new(1.)),
    );

    let limit_cond = Box::new(Wall {
        x_min: 0.,
        y_min: 0.,
        x_max: width as Scalar,
        y_max: height as Scalar,
        restitution: 0.8,
    });

    let gravity = Box::new(UniformGravity::new(Vector2::new(0., -0.003)));

    let physics = Box::new(Physics::new(
        vec![gravity],
        Box::new(GaussianIntegrator::new()),
    ));

    let mut particles = Particles::new_empty();
    factory.create(500_000, &mut particles);

    let sim = Simulation::new(particles, vec![limit_cond, physics]);

    let sim_runner = Box::new(ContinuousSimulationRunner::new(1.));

    let renderer = Box::new(BasicRenderer::new(get_window(width, height), None));

    let main = IridiumMain::new(
        sim,
        renderer,
        sim_runner,
        Box::new(default_event_handler),
        Duration::from_secs(1),
        1,
    );

    main
}

pub fn fireworks(width: u32, height: u32) -> IridiumMain {
    let limit_cond = Box::new(Wall {
        x_min: 0.,
        y_min: 0.,
        x_max: width as Scalar,
        y_max: height as Scalar,
        restitution: 0.8,
    });

    let gravity = Box::new(UniformGravity::new(Vector2::new(0., -0.001)));

    let physics = Box::new(Physics::new(
        vec![gravity],
        Box::new(GaussianIntegrator::new()),
    ));

    let sim = Simulation::new(Particles::new_empty(), vec![limit_cond, physics]);

    let sim_runner = Box::new(ContinuousSimulationRunner::new(1.));

    let event_handler = |m_renderer: &mut Box<dyn Renderer>,
                         m_sim: &mut Simulation,
                         running: &mut bool,
                         &event: &SfmlEvent| match event {
        SfmlEvent::MouseButtonPressed {
            button: sfml::window::mouse::Button::Left,
            x,
            y,
            ..
        } => {
            let seed: u64 = 0;
            let mut pfactory = GeneratorFactory::new(
                Box::new(PointGenerator::new(Point {
                    position: m_renderer.screen2sim(Vector2f::new(x as f32, y as f32)),
                })),
                Box::new(Vector2PolarGenerator::new(
                    Box::new(UniformGenerator::new(
                        XorShiftRng::seed_from_u64(seed),
                        0.,
                        1.,
                    )),
                    Box::new(UniformGenerator::new(
                        XorShiftRng::seed_from_u64(seed),
                        0.,
                        2. * PI,
                    )),
                )),
                Box::new(ConstantGenerator::new(1.)),
            );

            pfactory.create(1_000, &mut m_sim.particles);
        }
        _ => default_event_handler(m_renderer, m_sim, running, &event),
    };

    let renderer = Box::new(BasicRenderer::new(get_window(width, height), None));

    let main = IridiumMain::new(
        sim,
        renderer,
        sim_runner,
        Box::new(event_handler),
        Duration::from_secs(1),
        1,
    );

    main
}

pub fn flow(width: u32, height: u32) -> IridiumMain {
    let seed: u64 = 0;

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
                XorShiftRng::seed_from_u64(seed),
            )),
            Box::new(Vector2PolarGenerator::new(
                Box::new(ConstantGenerator::new(0.5)),
                Box::new(ConstantGenerator::new(0.1 * PI)),
            )),
            Box::new(ConstantGenerator::new(1.)),
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

    let mut events = SortedVec::new();
    events.add(Event::new(
        5000.,
        Box::new(|particles| {
            for vel in particles.velocities.iter_mut() {
                *vel *= 0.5;
            }
        }),
    ));

    let events_handler = Box::new(EventsHandler::new(events, 0.));

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
        Box::new(GaussianIntegrator::new()),
    ));

    let sim = Simulation::new(
        Particles::new_empty(),
        vec![emitter, consumer, events_handler, limit_cond, physics],
    );

    let sim_runner = Box::new(ContinuousSimulationRunner::new(1.));

    let renderer = Box::new(BasicRenderer::new(get_window(width, height), None));

    let main = IridiumMain::new(
        sim,
        renderer,
        sim_runner,
        Box::new(default_event_handler),
        Duration::from_secs(1),
        4,
    );

    main
}

struct SimReset;

impl System for SimReset {
    fn update(&mut self, particles: &mut Particles, _dt: Scalar) {
        particles.clear();
    }
}

pub fn benchmark2() -> IridiumMain {
    let seed: u64 = 0;
    let width = 500;
    let height = 500;

    let area = Rect {
        position: Vector2::new(0., 0.),
        size: Vector2::new(width as Scalar, height as Scalar),
    };

    let emitter = Box::new(ConstantEmitter::new(
        Box::new(GeneratorFactory::new(
            Box::new(RectGenerator::new(area, XorShiftRng::seed_from_u64(seed))),
            Box::new(Vector2PolarGenerator::new(
                Box::new(ConstantGenerator::new(0.5)),
                Box::new(ConstantGenerator::new(0.1 * PI)),
            )),
            Box::new(ConstantGenerator::new(1.)),
        )),
        5E5,
    ));

    let sim_reseter = Box::new(SimReset);

    let sim = Simulation::new(Particles::new_empty(), vec![sim_reseter, emitter]);

    let sim_runner = Box::new(ContinuousSimulationRunner::new(1.));

    let renderer = Box::new(BasicRenderer::new(get_window(width, height), None));

    let main = IridiumMain::new(
        sim,
        renderer,
        sim_runner,
        Box::new(default_event_handler),
        Duration::from_secs(1),
        4,
    );

    main
}

// pub fn events() {}
