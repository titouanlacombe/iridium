use criterion::{criterion_group, criterion_main, Criterion};
use nalgebra::Vector2;
use std::time::Duration;

use iridium::{
    examples::gen_planet,
    simulation::{
        areas::Rect, color::Color, forces::Gravity, particles::Particles, quadtree::QuadTree,
        random::RngGenerator,
    },
};

fn generate_particles(n: usize) -> Particles {
    let mut rng_gen = RngGenerator::new(0);
    let mut particles = Particles::new_empty();

    gen_planet(
        Vector2::new(500., 500.),
        100.,
        1.,
        Color::BLACK,
        n,
        &mut rng_gen,
        &mut particles,
    );

    particles
}

fn benchmark_qt(c: &mut Criterion) {
    let mut group = c.benchmark_group("quadtree");
    group.warm_up_time(Duration::from_millis(200));
    group.measurement_time(Duration::from_secs(2));

    let particles = generate_particles(1000);

    group.bench_function("insertion", |b| {
        b.iter(|| {
            let mut quadtree = QuadTree::new(
                Rect::new(Vector2::new(0.0, 0.0), Vector2::new(1000.0, 1000.0)),
                10,
                Gravity::new(1., 0.),
                0.5,
            );
            quadtree.insert_particles(&particles);
        })
    });

    let mut quadtree = QuadTree::new(
        Rect::new(Vector2::new(0.0, 0.0), Vector2::new(1000.0, 1000.0)),
        10,
        Gravity::new(1., 0.),
        0.5,
    );

    group.bench_function("re-insertion", |b| {
        b.iter(|| {
            quadtree.insert_particles(&particles);
        })
    });

    let mut forces = vec![Vector2::new(0.0, 0.0); particles.len()];

    group.bench_function("barnes_hut", |b| {
        b.iter(|| {
            quadtree.barnes_hut_particles(&particles, &mut forces);
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_qt);
criterion_main!(benches);
