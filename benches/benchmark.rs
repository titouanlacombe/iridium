use criterion::{criterion_group, criterion_main, Criterion};
use nalgebra::Vector2;
use rayon::prelude::*;
use std::time::Duration;

use iridium::{
    examples::gen_planet,
    simulation::{
        areas::Rect,
        color::Color,
        forces::{Drag, Force, Gravity, Repulsion, UniformDrag, UniformGravity},
        integrator::{GaussianIntegrator, Integrator},
        particles::Particles,
        quadtree::QuadTree,
        systems::{Physics, System, VelocityIntegrator},
    },
};

fn generate_particles(n: usize) -> Particles {
    let mut particles = Particles::new_empty();

    gen_planet(
        Vector2::new(500., 500.),
        Vector2::new(0., 0.),
        500.,
        1.,
        Color::BLACK,
        n,
        &mut particles,
    );

    particles
}

// TODO add benchmark for more parameters (theta, max_particles)
fn benchmark_qt(c: &mut Criterion) {
    let mut group = c.benchmark_group("quadtree");
    group.warm_up_time(Duration::from_millis(400));
    group.measurement_time(Duration::from_secs(4));

    // Start the Tracy client
    tracy_client::Client::start();

    let particles = generate_particles(3000);
    let max_particles = 100;
    let theta = 0.5;
    let gravity = Gravity::new(1., 0.);
    let repulsion = Repulsion::new(1., 6, 0.);
    let drag = Drag::new(1., 0.);
    let rect = Rect::new(Vector2::new(0.0, 0.0), Vector2::new(1000.0, 1000.0));

    group.bench_function("insertion", |b| {
        b.iter(|| {
            let mut quadtree = QuadTree::new(
                rect.clone(),
                max_particles,
                gravity.clone(),
                repulsion.clone(),
                drag.clone(),
                theta,
                None,
                false,
            );
            quadtree.insert_particles(&particles);
        })
    });

    let mut quadtree = QuadTree::new(
        rect,
        max_particles,
        gravity.clone(),
        repulsion.clone(),
        drag.clone(),
        theta,
        None,
        false,
    );
    quadtree.insert_particles(&particles);

    group.bench_function("re-insertion", |b| {
        b.iter(|| {
            quadtree.insert_particles(&particles);
        })
    });

    let mut forces = vec![Vector2::new(0.0, 0.0); particles.len()];

    group.bench_function("naive", |b| {
        b.iter(|| {
            gravity.clone().apply(&particles, &mut forces);
            repulsion.clone().apply(&particles, &mut forces);
            drag.clone().apply(&particles, &mut forces);
        })
    });

    group.bench_function("barnes_hut", |b| {
        b.iter(|| {
            quadtree.barnes_hut_particles(&particles, &mut forces);
        })
    });

    group.finish();
}

fn benchmark_buffer_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_ops");
    group.warm_up_time(Duration::from_millis(400));
    group.measurement_time(Duration::from_secs(4));

    let mut particles = generate_particles(100_000);
    let mut forces = vec![Vector2::zeros(); particles.len()];

    let mut uniform_gravity = UniformGravity::new(Vector2::new(0.0, -9.81));
    let mut uniform_drag = UniformDrag::new(0.1, Vector2::zeros());

    group.bench_function("uniform_gravity", |b| {
        b.iter(|| uniform_gravity.apply(&particles, &mut forces))
    });

    group.bench_function("uniform_drag", |b| {
        b.iter(|| uniform_drag.apply(&particles, &mut forces))
    });

    group.bench_function("mass_divide", |b| {
        b.iter(|| {
            forces
                .par_iter_mut()
                .zip(particles.masses.par_iter())
                .for_each(|(force, mass)| *force /= *mass);
        })
    });

    // Constant dt: benches must not depend on variable render frame time
    let dt = 1.0 / 120.0;
    let integrator = GaussianIntegrator;
    let mut positions_out = particles.positions.clone();
    group.bench_function("integrate", |b| {
        b.iter(|| {
            integrator.integrate_vec(&particles.velocities, &mut positions_out, dt);
        })
    });

    let mut physics = Physics::new(
        vec![
            Box::new(UniformGravity::new(Vector2::new(0.0, -9.81))),
            Box::new(UniformDrag::new(0.1, Vector2::zeros())),
        ],
        Box::new(GaussianIntegrator),
    );
    let mut velocity_integrator = VelocityIntegrator::new(Box::new(GaussianIntegrator));

    group.bench_function("full_step", |b| {
        b.iter(|| {
            physics.update(&mut particles, dt);
            velocity_integrator.update(&mut particles, dt);
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_qt, benchmark_buffer_ops);
criterion_main!(benches);
