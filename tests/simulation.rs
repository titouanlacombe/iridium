use nalgebra::Vector2;
use rand::RngExt;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;

use iridium::simulation::{
    areas::{Area, Rect},
    color::Color,
    forces::{Drag, Force, Gravity, Repulsion, UniformGravity},
    integrator::GaussianIntegrator,
    particles::Particles,
    quadtree::QuadTree,
    systems::{Physics, System, VelocityIntegrator},
};

fn generate_random_particles(n: usize, seed: u64, size: f64) -> Particles {
    let mut rng = Pcg64Mcg::seed_from_u64(seed);

    let positions: Vec<Vector2<f64>> = (0..n)
        .map(|_| Vector2::new(rng.random_range(0.0..size), rng.random_range(0.0..size)))
        .collect();
    let velocities: Vec<Vector2<f64>> = (0..n)
        .map(|_| Vector2::new(rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0)))
        .collect();
    let masses: Vec<f64> = (0..n).map(|_| rng.random_range(1.0..10.0)).collect();

    Particles::new(positions, velocities, masses, vec![Color::BLACK; n])
}

fn total_momentum(particles: &Particles) -> Vector2<f64> {
    particles
        .masses
        .iter()
        .zip(&particles.velocities)
        .map(|(mass, velocity)| *mass * velocity)
        .sum()
}

fn total_energy(particles: &Particles, g_coef: f64) -> f64 {
    let mut energy = 0.0;

    for i in 0..particles.len() {
        energy += 0.5 * particles.masses[i] * particles.velocities[i].norm_squared();
    }

    for i in 0..particles.len() {
        for j in (i + 1)..particles.len() {
            let distance = (particles.positions[i] - particles.positions[j]).norm();
            if distance > 0.0 {
                energy -= g_coef * particles.masses[i] * particles.masses[j] / distance;
            }
        }
    }

    energy
}

#[test]
fn force_symmetry() {
    let p1 = Vector2::new(1.0, 2.0);
    let p2 = Vector2::new(-3.5, 4.2);
    let v1 = Vector2::new(0.5, -0.2);
    let v2 = Vector2::new(-0.8, 0.3);
    let m1 = 3.0;
    let m2 = 7.0;

    let gravity = Gravity::new(1.0, 0.0);
    let repulsion = Repulsion::new(1.0, 6, 0.0);
    let drag = Drag::new(1.0, 10.0);

    let g12 = gravity.calc_force(p1, p2, m1, m2);
    let g21 = gravity.calc_force(p2, p1, m2, m1);
    assert!(
        (g12 + g21).norm() < 1e-12,
        "gravity not antisymmetric: {g12} vs {g21}"
    );

    let r12 = repulsion.calc_force(p1, p2);
    let r21 = repulsion.calc_force(p2, p1);
    assert!(
        (r12 + r21).norm() < 1e-12,
        "repulsion not antisymmetric: {r12} vs {r21}"
    );

    let d12 = drag.calc_force(p1, p2, v1, v2);
    let d21 = drag.calc_force(p2, p1, v2, v1);
    assert!(
        (d12 + d21).norm() < 1e-12,
        "drag not antisymmetric: {d12} vs {d21}"
    );
}

#[test]
fn barnes_hut_matches_naive_at_theta_zero() {
    let n = 500;
    let particles = generate_random_particles(n, 7, 1000.0);
    let mut forces_naive = vec![Vector2::zeros(); n];
    let mut forces_bh = vec![Vector2::zeros(); n];

    let gravity = Gravity::new(1.0, 0.0);
    let repulsion = Repulsion::new(1.0, 6, 0.0);
    let drag = Drag::new(1.0, 50.0);

    gravity.clone().apply(&particles, &mut forces_naive);
    repulsion.clone().apply(&particles, &mut forces_naive);
    drag.clone().apply(&particles, &mut forces_naive);

    let rect = Rect::new(Vector2::new(0.0, 0.0), Vector2::new(1000.0, 1000.0));
    let mut quadtree = QuadTree::new(rect, 16, gravity, repulsion, drag, 0.0, None, false);
    quadtree.barnes_hut_particles(&particles, &mut forces_bh);

    for i in 0..n {
        let diff = (forces_naive[i] - forces_bh[i]).norm();
        let scale = forces_naive[i].norm().max(forces_bh[i].norm()).max(1e-30);
        assert!(
            diff / scale < 1e-6,
            "particle {i}: naive {:?} vs bh {:?}",
            forces_naive[i],
            forces_bh[i]
        );
    }
}

#[test]
fn momentum_and_energy_conserved_with_nbody_gravity() {
    let g_coef = 0.05;
    let dt = 0.005;
    let steps = 200;

    // Massive central particle at rest + test particle in a circular orbit.
    // Close encounters are avoided so the force stays smooth.
    let central_mass = 1000.0;
    let test_mass = 1.0;
    let radius = 5.0_f64;
    let orbital_speed = (g_coef * central_mass / radius).sqrt();

    let mut particles = Particles::new(
        vec![Vector2::zeros(), Vector2::new(radius, 0.0)],
        vec![Vector2::zeros(), Vector2::new(0.0, orbital_speed)],
        vec![central_mass, test_mass],
        vec![Color::BLACK; 2],
    );

    let momentum0 = total_momentum(&particles);
    let energy0 = total_energy(&particles, g_coef);

    let mut physics = Physics::new(
        vec![Box::new(Gravity::new(g_coef, 0.0))],
        Box::new(GaussianIntegrator),
    );
    let mut velocity_integrator = VelocityIntegrator::new(Box::new(GaussianIntegrator));

    for _ in 0..steps {
        physics.update(&mut particles, dt);
        velocity_integrator.update(&mut particles, dt);
    }

    let momentum1 = total_momentum(&particles);
    let energy1 = total_energy(&particles, g_coef);

    assert!(
        (momentum1 - momentum0).norm() / momentum0.norm() < 1e-9,
        "momentum drifted: {momentum0} -> {momentum1}"
    );
    assert!(
        (energy1 - energy0).abs() / energy0.abs() < 1e-2,
        "energy drifted: {energy0} -> {energy1}"
    );
    assert!(
        (particles.positions[1].norm() - radius).abs() < 0.5,
        "orbit radius drifted: {}",
        particles.positions[1].norm()
    );
}

#[test]
fn uniform_gravity_matches_analytic_solution() {
    let g = Vector2::new(0.0, -9.81);

    for (dt, steps) in [(0.01, 100), (1.0, 10)] {
        let mut particles = Particles::new(
            vec![Vector2::zeros()],
            vec![Vector2::zeros()],
            vec![1.0],
            vec![Color::BLACK],
        );

        let mut physics = Physics::new(
            vec![Box::new(UniformGravity::new(g))],
            Box::new(GaussianIntegrator),
        );
        let mut velocity_integrator = VelocityIntegrator::new(Box::new(GaussianIntegrator));

        for _ in 0..steps {
            physics.update(&mut particles, dt);
            velocity_integrator.update(&mut particles, dt);
        }

        // Velocity-leading symplectic Euler: x_n = g * dt^2 * n*(n+1)/2
        let expected_pos = g * dt * dt * (steps * (steps + 1) / 2) as f64;
        let expected_vel = g * dt * steps as f64;

        assert!(
            (particles.positions[0] - expected_pos).norm() < 1e-9,
            "dt={dt}: pos {:?} vs expected {:?}",
            particles.positions[0],
            expected_pos
        );
        assert!(
            (particles.velocities[0] - expected_vel).norm() < 1e-9,
            "dt={dt}: vel {:?} vs expected {:?}",
            particles.velocities[0],
            expected_vel
        );
    }
}

#[test]
fn quadtree_structure_invariants() {
    let n = 1000;
    let max_particles = 4;
    let particles = generate_random_particles(n, 3, 1000.0);

    let rect = Rect::new(Vector2::new(0.0, 0.0), Vector2::new(1000.0, 1000.0));
    let mut quadtree = QuadTree::new(
        rect,
        max_particles,
        Gravity::new(1.0, 0.0),
        Repulsion::new(1.0, 6, 0.0),
        Drag::new(1.0, 50.0),
        0.5,
        None,
        false,
    );
    let (max_depth, nodes) = quadtree.insert_particles(&particles);

    assert!(max_depth > 0);
    assert!(nodes >= n / max_particles);

    // Every particle index must appear in exactly one leaf, inside its rect
    let mut seen = vec![false; n];
    let mut leaf_count = 0;
    let mut stack = vec![&quadtree.root];

    while let Some(node) = stack.pop() {
        if node.childs.is_empty() {
            leaf_count += 1;
            assert!(node.indexes.len() <= max_particles);
            assert_eq!(node.indexes.len(), node.particles.len());

            let mut center_of_mass = Vector2::zeros();
            let mut total_mass = 0.0;

            for (&i, pos) in node.indexes.iter().zip(&node.particles.positions) {
                assert!(!seen[i], "index {i} in multiple leaves");
                seen[i] = true;
                assert!(node.rect.contain(particles.positions[i]));
                assert!((particles.positions[i] - pos).norm() < 1e-12);

                let mass = particles.masses[i];
                center_of_mass += pos * mass;
                total_mass += mass;
            }

            assert!((node.total_mass - total_mass).abs() < 1e-9);
            if total_mass > 0.0 {
                assert!((node.center_of_mass - center_of_mass / total_mass).norm() < 1e-9);
            }
        } else {
            assert_eq!(node.indexes.len(), 0);
            stack.extend(node.childs.iter());
        }
    }

    assert!(seen.iter().all(|&s| s), "some particles missing from leaves");
    assert!(leaf_count > 1);
}
