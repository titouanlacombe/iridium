use criterion::{criterion_group, criterion_main, Criterion};
use nalgebra::Vector2;
use std::time::Duration;

use iridium::{
    examples::gen_planet,
    simulation::{
        areas::Rect,
        color::Color,
        forces::{Drag, Force, Gravity, Repulsion},
        particles::Particles,
        quadtree::QuadTree,
        random::RngGenerator,
    },
};

fn generate_particles(n: usize) -> Particles {
    let mut rng_gen = RngGenerator::new(0);
    let mut particles = Particles::new_empty();

    gen_planet(
        Vector2::new(500., 500.),
        Vector2::new(0., 0.),
        500.,
        1.,
        Color::BLACK,
        n,
        &mut rng_gen,
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

criterion_group!(benches, benchmark_qt);
criterion_main!(benches);
