use nalgebra::Vector2;

use crate::{
    areas::Disk,
    events::{Event, EventsHandler, SortedVec},
    forces::UniformGravity,
    particle::{ParticleFactory, RandomFactory},
    simulation::{ContinuousSimulationRunner, LimitCond, Simulation, SimulationRunner},
    updatables::{ConstantConsumer, ConstantEmitter},
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

    let simulation = Simulation::new(
        particles,
        vec![],
        Some(UniformGravity::new(Vector2::new(0., -0.003))),
        None,
        LimitCond::Wall(0., 0., 500., 500., 0.8),
    );

    let sim_runner = Box::new(ContinuousSimulationRunner::new(simulation, 1.));

    sim_runner
}

pub fn fireworks(width: u32, height: u32) -> Box<dyn SimulationRunner> {
    // TODO define key handler here
    let simulation = Simulation::new(
        Vec::new(),
        vec![],
        Some(UniformGravity::new(Vector2::new(0., -0.001))),
        None,
        LimitCond::Wall(0., 0., width as f32, height as f32, 0.8),
    );

    let sim_runner = Box::new(ContinuousSimulationRunner::new(simulation, 1.));

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

    let simulation = Simulation::new(
        Vec::new(),
        vec![emitter, consumer, events_handler],
        Some(UniformGravity::new(Vector2::new(0., -0.001))),
        None,
        LimitCond::Wall(0., 0., width as f32, height as f32, 0.8),
    );

    let sim_runner = Box::new(ContinuousSimulationRunner::new(simulation, 1.));

    sim_runner
}
