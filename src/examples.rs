use nalgebra::Vector2;
use rayon::prelude::*;
use sfml::{
    system::Vector2f,
    window::{Event as SfmlEvent, Key},
};
use std::{
    cmp::max,
    f64::consts::PI,
    ops::Deref,
    sync::{Arc, RwLock},
    time::Duration,
};

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
        color::Color,
        forces::{Drag, Gravity, Repulsion, UniformDrag, UniformGravity},
        generators::{
            ConstantGenerator, DiskGenerator, HSVAGenerator, PointGenerator, RectGenerator,
            UniformGenerator, Vector2PolarGenerator,
        },
        integrator::GaussianIntegrator,
        particles::{GeneratorFactory, ParticleFactory, Particles},
        quadtree::{QuadTree, QuadtreeForces},
        random::RngGenerator,
        sim_events::{DefaultSimEventsHandler, SimEvent},
        simulation::{ConstantSimulationRunner, Simulation, SimulationRunner},
        systems::{
            ColorWheel, ConstantConsumer, ConstantEmitter, Physics, System, VelocityIntegrator,
            Wall,
        },
        types::Scalar,
    },
    utils::sorted_vec::SortedVec,
};

fn get_default_input_callback() -> InputCallback {
    let mut keys_state = KeysState::new();
    let mut single_step = false;

    Box::new(move |data, render_data, dt, events| {
        let view_data = &mut render_data.view_data.write().unwrap();

        if single_step {
            single_step = false;
            data.running = false;
        }

        for event in events {
            keys_state.update(&event);

            match event.original {
                // TODO: Key to disable/enable quadtree rendering
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
                    Key::S | Key::N => {
                        data.running = true;
                        single_step = true;
                    }
                    _ => {}
                },
                SfmlEvent::MouseWheelScrolled { delta, .. } => {
                    view_data.zoom *= 1. - delta * 0.05;
                }
                SfmlEvent::Resized { width, height } => {
                    view_data.size = Vector2f::new(width as f32, height as f32);
                }
                _ => {}
            }
        }

        let translation_speed = (dt * 200. * view_data.zoom as f64) as f32;
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
    quadtree: Option<Arc<RwLock<QuadTree>>>,
) -> AppMain {
    let view_data = ViewData::new(
        Vector2f::new(width as f32 / 2., height as f32 / 2.),
        Vector2f::new(width as f32, height as f32),
        sfml::graphics::FloatRect::new(0., 0., 1., 1.),
        0.,
        1.,
    );

    let render_data = RenderData::new(view_data);

    let render_thread = RenderThread::new(WindowData::new(
        (width, height),
        format!("Iridium - {}", sim_name),
        sfml::window::Style::DEFAULT,
        sfml::window::ContextSettings::default(),
        false,
    ));

    AppMain::new(
        sim,
        Box::new(BasicRenderer::new(
            quadtree,
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

pub fn benchmark_empty() -> AppMain {
    let width = 500;
    let height = 500;

    let sim = Simulation::new(Particles::new_empty(), vec![], None);

    let sim_runner = Box::new(ConstantSimulationRunner::new(1.));

    base_iridium_app(
        width,
        height,
        sim,
        sim_runner,
        "Benchmark Empty",
        None,
        get_default_input_callback(),
        None,
    )
}

pub fn benchmark_base() -> AppMain {
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
        Box::new(ConstantGenerator::new(Color::WHITE)),
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

    let sim_runner = Box::new(ConstantSimulationRunner::new(1.));

    base_iridium_app(
        width,
        height,
        sim,
        sim_runner,
        "Benchmark Base",
        None,
        get_default_input_callback(),
        None,
    )
}

pub fn benchmark_forces() -> AppMain {
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
        Box::new(ConstantGenerator::new(Color::WHITE)),
    );

    let mut particles = Particles::new_empty();
    factory.create(1_000_000, &mut particles);

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

    let sim_runner = Box::new(ConstantSimulationRunner::new(1.));

    base_iridium_app(
        width,
        height,
        sim,
        sim_runner,
        "Benchmark Forces",
        None,
        get_default_input_callback(),
        None,
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

    let sim_runner = Box::new(ConstantSimulationRunner::new(1.));

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
                        let mut firework_factory = GeneratorFactory::new(
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

                        firework_factory.create(1_000, &mut data.sim.particles);
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
        "Fireworks",
        max_fps(60),
        input_callback,
        None,
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
            Box::new(ConstantGenerator::new(Color::RED)),
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

    let color_wheel = Box::new(ColorWheel { speed: 0.2 });

    let systems: Vec<Box<dyn System>> = vec![
        emitter,
        consumer,
        limit_cond,
        physics,
        velocity_integrator,
        color_wheel,
    ];

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

    let sim_runner = Box::new(ConstantSimulationRunner::new(4.));

    base_iridium_app(
        width,
        height,
        sim,
        sim_runner,
        "Flow",
        max_fps(60),
        get_default_input_callback(),
        None,
    )
}

struct SimReset;

impl System for SimReset {
    fn update(&mut self, particles: &mut Particles, _dt: Scalar) {
        particles.clear();
    }
}

pub fn benchmark_generator() -> AppMain {
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
            Box::new(ConstantGenerator::new(Color::WHITE)),
        )),
        5E5,
    ));

    let sim_reseter = Box::new(SimReset);

    let sim = Simulation::new(Particles::new_empty(), vec![sim_reseter, emitter], None);

    let sim_runner = Box::new(ConstantSimulationRunner::new(1.));

    base_iridium_app(
        width,
        height,
        sim,
        sim_runner,
        "Benchmark Generator",
        None,
        get_default_input_callback(),
        None,
    )
}

// pub fn events() {}

// Temporary facade to generate a planet
// TODO improve algorithm to avoid overlapping (add method to disk, Area.uniform(n)?)
pub fn gen_planet(
    position: Vector2<Scalar>,
    velocity: Vector2<Scalar>,
    radius: Scalar,
    mass: Scalar,
    color: Color,
    n: usize,
    rng_gen: &mut RngGenerator,
    particles: &mut Particles,
) {
    GeneratorFactory::new(
        Box::new(DiskGenerator::new(
            Disk::new(position, radius),
            rng_gen.next(),
        )),
        Box::new(ConstantGenerator::new(velocity)),
        Box::new(ConstantGenerator::new(mass / n as Scalar)),
        Box::new(ConstantGenerator::new(color)),
    )
    .create(n, particles);
}

pub fn benchmark_gravity() -> AppMain {
    let width = 1200;
    let height = 800;
    let dt = 0.5;

    let mut rng_gen = RngGenerator::new(0);
    let sim_space = Rect::new(
        Vector2::new(0., 0.),
        Vector2::new(width as Scalar, height as Scalar),
    );
    let center = sim_space.center();

    let mut particles = Particles::new_empty();

    let offset = Vector2::new(450., 0.);
    let velocity = Vector2::new(0.45, -0.07);

    // Generate planet
    gen_planet(
        center + offset,
        -velocity,
        90.,
        1500.,
        Color::CYAN,
        1800,
        &mut rng_gen,
        &mut particles,
    );

    // Generate planet 2
    gen_planet(
        center - offset,
        velocity,
        80.,
        1500.,
        Color::YELLOW,
        1200,
        &mut rng_gen,
        &mut particles,
    );

    // Generate black hole
    // gen_planet(
    //     center - offset,
    //     velocity,
    //     0.,
    //     n as Scalar,
    //     Color::RED,
    //     1,
    //     &mut rng_gen,
    //     &mut particles,
    // );

    let limit_cond = Box::new(Wall {
        x_min: 0.,
        y_min: 0.,
        x_max: width as Scalar,
        y_max: height as Scalar,
        restitution: 0.,
    });

    let gravity = Box::new(Gravity::new(0.03, 3.));
    let repulsion = Box::new(Repulsion::new(10., 6, 1.5));
    let drag = Box::new(Drag::new(0.0013, 15.));

    // Quatree wraps simulation space
    let qt_size = max(width, height) as Scalar;
    let quadtree_rect = Rect::new(
        center - Vector2::new(qt_size / 2., qt_size / 2.),
        Vector2::new(qt_size, qt_size),
    );

    let quadtree = Arc::new(RwLock::new(QuadTree::new(
        quadtree_rect,
        10,
        gravity.deref().clone(),
        repulsion.deref().clone(),
        drag.deref().clone(),
        1.5,
    )));

    let quadtree_forces = Box::new(QuadtreeForces::new(quadtree.clone()));

    let physics = Box::new(Physics::new(
        vec![quadtree_forces],
        Box::new(GaussianIntegrator),
    ));

    let velocity_integrator = Box::new(VelocityIntegrator::new(Box::new(GaussianIntegrator)));

    let systems: Vec<Box<dyn System>> = vec![limit_cond, physics, velocity_integrator];

    let sim = Simulation::new(particles, systems, None);

    let sim_runner = Box::new(ConstantSimulationRunner::new(dt));

    base_iridium_app(
        width,
        height,
        sim,
        sim_runner,
        "Gravity",
        max_fps(144),
        get_default_input_callback(),
        Some(quadtree),
    )
}
