use nalgebra::Vector2;

use crate::{
    areas::Disk,
    events::{Event, EventsHandler, SortedVec},
    forces::UniformGravity,
    particle::{ParticleFactory, RandomFactory},
    simulation::{ContinuousSimulationRunner, Simulation, SimulationRunner},
    systems::{ConstantConsumer, ConstantEmitter, Wall},
};

pub fn benchmark1() -> Box<dyn SimulationRunner> {
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
        x_max: 500.,
        y_max: 500.,
        restitution: 0.8,
    });

    let simulation = Simulation::new(
        particles,
        vec![limit_cond],
        Some(UniformGravity::new(Vector2::new(0., -0.003))),
        None,
    );

    let sim_runner = Box::new(ContinuousSimulationRunner::new(simulation, 1.));

    sim_runner
}

pub fn fireworks(width: u32, height: u32) -> Box<dyn SimulationRunner> {
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

    // TODO define key handler here (examples output renderer)
    sim_runner
}

pub fn flow(width: u32, height: u32) -> Box<dyn SimulationRunner> {
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

    sim_runner
}
