use nalgebra::Vector2;
use sfml::{
    graphics::RenderWindow,
    system::Vector2f,
    window::{Event as SfmlEvent, Key},
};
use std::time::Duration;

use crate::{
    areas::{Disk, Point},
    events::{Event, EventsHandler, SortedVec},
    forces::UniformGravity,
    generators::{ConstantGenerator, UniformGenerator, Vector2PolarGenerator},
    iridium::{max_fps, IridiumMain},
    particle::{GeneratorFactory, ParticleFactory},
    renderer::BasicRenderer,
    simulation::{ContinuousSimulationRunner, Simulation},
    systems::{ConstantConsumer, ConstantEmitter, Wall},
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
    _renderer: &mut BasicRenderer,
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

    let factory = GeneratorFactory::new(
        Box::new(Disk {
            position: Vector2::new(200., 300.),
            radius: 100.,
        }),
        Box::new(Vector2PolarGenerator::new(
            Box::new(UniformGenerator::new(1.2, 1.5)),
            Box::new(UniformGenerator::new(-0.2 * std::f32::consts::PI, 0.)),
        )),
        Box::new(ConstantGenerator::new(1.)),
    );

    let limit_cond = Box::new(Wall {
        x_min: 0.,
        y_min: 0.,
        x_max: width as f32,
        y_max: height as f32,
        restitution: 0.8,
    });

    let sim = Simulation::new(
        factory.create(500_000),
        vec![limit_cond],
        Some(UniformGravity::new(Vector2::new(0., -0.003))),
        None,
    );

    let sim_runner = Box::new(ContinuousSimulationRunner::new(1.));

    let renderer = BasicRenderer::new(get_window(width, height), None);

    let main = IridiumMain::new(
        renderer,
        sim,
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
        x_max: width as f32,
        y_max: height as f32,
        restitution: 0.8,
    });

    let sim = Simulation::new(
        Vec::new(),
        vec![limit_cond],
        Some(UniformGravity::new(Vector2::new(0., -0.001))),
        None,
    );

    let sim_runner = Box::new(ContinuousSimulationRunner::new(1.));

    let event_handler = |m_renderer: &mut BasicRenderer,
                         m_sim: &mut Simulation,
                         running: &mut bool,
                         &event: &SfmlEvent| match event {
        SfmlEvent::MouseButtonPressed {
            button: sfml::window::mouse::Button::Left,
            x,
            y,
            ..
        } => {
            let pfactory = GeneratorFactory::new(
                Box::new(Point {
                    position: m_renderer.screen2sim(Vector2f::new(x as f32, y as f32)),
                }),
                Box::new(Vector2PolarGenerator::new(
                    Box::new(UniformGenerator::new(0., 1.)),
                    Box::new(UniformGenerator::new(0., 2. * std::f32::consts::PI)),
                )),
                Box::new(ConstantGenerator::new(1.)),
            );

            let new = pfactory.create(1_000);
            m_sim.particles.extend(new);
        }
        _ => default_event_handler(m_renderer, m_sim, running, &event),
    };

    let renderer = BasicRenderer::new(get_window(width, height), max_fps(400));

    let main = IridiumMain::new(
        renderer,
        sim,
        sim_runner,
        Box::new(event_handler),
        Duration::from_secs(1),
        1,
    );

    main
}

pub fn flow(width: u32, height: u32) -> IridiumMain {
    let emitter = Box::new(ConstantEmitter::new(
        Box::new(GeneratorFactory::new(
            Box::new(Disk {
                position: Vector2::new(width as f32 / 10., height as f32 - (height as f32 / 10.)),
                radius: width as f32 / 10.,
            }),
            Box::new(Vector2PolarGenerator::new(
                Box::new(UniformGenerator::new(0.4, 0.4)),
                Box::new(UniformGenerator::new(0., 0.2 * std::f32::consts::PI)),
            )),
            Box::new(ConstantGenerator::new(1.)),
        )),
        30.,
    ));

    let consumer = Box::new(ConstantConsumer::new(
        Box::new(Disk {
            position: Vector2::new(width as f32 / 2., height as f32 / 2.),
            radius: width as f32 / 10.,
        }),
        3.,
    ));

    let mut events = SortedVec::new();
    events.add(Event::new(
        5000.,
        Box::new(|particles| {
            for particle in particles.iter_mut() {
                particle.velocity = Vector2::new(0., 0.);
            }
        }),
    ));

    let events_handler = Box::new(EventsHandler::new(events, 0.));

    let limit_cond = Box::new(Wall {
        x_min: 0.,
        y_min: 0.,
        x_max: width as f32,
        y_max: height as f32,
        restitution: 0.8,
    });

    let sim = Simulation::new(
        Vec::new(),
        vec![emitter, consumer, events_handler, limit_cond],
        Some(UniformGravity::new(Vector2::new(0., -0.001))),
        None,
    );

    let sim_runner = Box::new(ContinuousSimulationRunner::new(1.));

    let renderer = BasicRenderer::new(get_window(width, height), None);

    let main = IridiumMain::new(
        renderer,
        sim,
        sim_runner,
        Box::new(default_event_handler),
        Duration::from_secs(1),
        4,
    );

    main
}
