use nalgebra::Vector2;
use sfml::graphics::RenderWindow;
use std::time::Duration;

use crate::{
    areas::Disk,
    events::{Event, EventsHandler, SortedVec},
    forces::UniformGravity,
    particle::{ParticleFactory, RandomFactory},
    renderer::IridiumRenderer,
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

pub fn benchmark1() -> IridiumRenderer {
    let width = 500;
    let height = 500;

    let factory = Box::new(RandomFactory::new(
        Box::new(Disk {
            position: Vector2::new(200., 300.),
            radius: 100.,
        }),
        1.2,
        1.5,
        -0.2 * std::f32::consts::PI,
        0.,
        1.,
        1.,
    ));

    let mut particles = Vec::new();
    for _ in 0..100_000 {
        particles.push(factory.create());
    }

    let limit_cond = Box::new(Wall {
        x_min: 0.,
        y_min: 0.,
        x_max: width as f32,
        y_max: height as f32,
        restitution: 0.8,
    });

    let simulation = Simulation::new(
        particles,
        vec![limit_cond],
        Some(UniformGravity::new(Vector2::new(0., -0.003))),
        None,
    );

    let sim_runner = Box::new(ContinuousSimulationRunner::new(simulation, 1.));

    let renderer = IridiumRenderer::new(
        get_window(width, height),
        sim_runner,
        None,
        Duration::from_secs(1),
    );

    renderer
}

pub fn fireworks(width: u32, height: u32) -> IridiumRenderer {
    let limit_cond = Box::new(Wall {
        x_min: 0.,
        y_min: 0.,
        x_max: width as f32,
        y_max: height as f32,
        restitution: 0.8,
    });

    let simulation = Simulation::new(
        Vec::new(),
        vec![limit_cond],
        Some(UniformGravity::new(Vector2::new(0., -0.001))),
        None,
    );

    let sim_runner = Box::new(ContinuousSimulationRunner::new(simulation, 1.));

    let renderer = IridiumRenderer::new(
        get_window(width, height),
        sim_runner,
        None,
        Duration::from_secs(1),
    );

    renderer
}

pub fn flow(width: u32, height: u32) -> IridiumRenderer {
    let emitter = Box::new(ConstantEmitter::new(
        Box::new(RandomFactory::new(
            Box::new(Disk {
                position: Vector2::new(width as f32 / 10., height as f32 - (height as f32 / 10.)),
                radius: width as f32 / 10.,
            }),
            0.4,
            0.4,
            0.,
            0.2 * std::f32::consts::PI,
            1.,
            1.,
        )),
        10.,
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

    let simulation = Simulation::new(
        Vec::new(),
        vec![emitter, consumer, events_handler, limit_cond],
        Some(UniformGravity::new(Vector2::new(0., -0.001))),
        None,
    );

    let sim_runner = Box::new(ContinuousSimulationRunner::new(simulation, 1.));

    let renderer = IridiumRenderer::new(
        get_window(width, height),
        sim_runner,
        None,
        Duration::from_secs(1),
    );

    renderer
}
