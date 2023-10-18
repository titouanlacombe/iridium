use criterion::{criterion_group, criterion_main, Criterion};
use nalgebra::Vector2;

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

fn benchmark_barnes_hut(c: &mut Criterion) {
    c.bench_function("barnes_hut", |b| {
        let particles = generate_particles(1000);
        let mut forces = vec![Vector2::new(0.0, 0.0); particles.len()];
        b.iter(|| {
            let mut quadtree = QuadTree::new(
                Rect::new(Vector2::new(0.0, 0.0), Vector2::new(1000.0, 1000.0)),
                10,
                Gravity::new(1., 0.),
                0.5,
            );
            quadtree.gravity(&particles, &mut forces);
        })
    });
}

criterion_group!(benches, benchmark_barnes_hut);
criterion_main!(benches);
