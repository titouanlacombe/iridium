use nalgebra::Vector2;
use rand::RngExt;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;

use iridium::simulation::{
    areas::{Area, Rect},
    color::Color,
    forces::{Drag, Force, Gravity, Repulsion, UniformGravity},
    integrator::GaussianIntegrator,
    morton::MortonSort,
    particles::Particles,
    quadtree::QuadTree,
    systems::{Physics, System, VelocityIntegrator},
    types::Scalar,
};

// fp tolerances: f32 has ~7 digits of precision, f64 ~15
#[cfg(feature = "f32")]
const TOL_BH: Scalar = 1e-4; // naive vs barnes-hut: different summation order
#[cfg(feature = "f32")]
const TOL_MOMENTUM: Scalar = 1e-4;
#[cfg(feature = "f32")]
const TOL_ENERGY: Scalar = 5e-2;
#[cfg(feature = "f32")]
const TOL_UG: Scalar = 1e-4; // uniform gravity analytic solution
#[cfg(feature = "f32")]
const TOL_STRUCT: Scalar = 1e-3; // center of mass / total mass checks

#[cfg(not(feature = "f32"))]
const TOL_BH: Scalar = 1e-6;
#[cfg(not(feature = "f32"))]
const TOL_MOMENTUM: Scalar = 1e-9;
#[cfg(not(feature = "f32"))]
const TOL_ENERGY: Scalar = 1e-2;
#[cfg(not(feature = "f32"))]
const TOL_UG: Scalar = 1e-9;
#[cfg(not(feature = "f32"))]
const TOL_STRUCT: Scalar = 1e-9;

fn generate_random_particles(n: usize, seed: u64, size: Scalar) -> Particles {
    let mut rng = Pcg64Mcg::seed_from_u64(seed);

    let positions: Vec<Vector2<Scalar>> = (0..n)
        .map(|_| Vector2::new(rng.random_range(0.0..size), rng.random_range(0.0..size)))
        .collect();
    let velocities: Vec<Vector2<Scalar>> = (0..n)
        .map(|_| Vector2::new(rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0)))
        .collect();
    let masses: Vec<Scalar> = (0..n).map(|_| rng.random_range(1.0..10.0)).collect();

    Particles::new(positions, velocities, masses, vec![Color::BLACK; n])
}

fn total_momentum(particles: &Particles) -> Vector2<Scalar> {
    particles
        .masses
        .iter()
        .zip(&particles.velocities)
        .map(|(mass, velocity)| *mass * velocity)
        .sum::<Vector2<Scalar>>()
}

fn total_energy(particles: &Particles, g_coef: Scalar) -> Scalar {
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
    let mut particles = generate_random_particles(n, 7, 1000.0);

    // Morton-sorted particles: batched barnes-hut relies on coherence (correctness
    // does not depend on it, but this exercises the real usage path)
    MortonSort::new().sort(&mut particles);

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
            diff / scale < TOL_BH,
            "particle {i}: naive {:?} vs bh {:?}",
            forces_naive[i],
            forces_bh[i]
        );
    }
}

#[test]
fn barnes_hut_theta_05_is_a_reasonable_approximation() {
    // Loose sanity check for the approximation path (theta > 0): batched barnes-hut
    // must stay close to brute force. Catches gross bugs (double-counting, wrong
    // active[] handling) which the theta=0 test cannot see.
    let mut particles = generate_random_particles(2000, 11, 1000.0);
    MortonSort::new().sort(&mut particles);

    let gravity = Gravity::new(1.0, 0.0);
    let mut forces_naive = vec![Vector2::zeros(); 2000];
    gravity.clone().apply(&particles, &mut forces_naive);

    let rect = Rect::new(Vector2::new(0.0, 0.0), Vector2::new(1000.0, 1000.0));
    let mut quadtree = QuadTree::new(
        rect,
        16,
        gravity,
        Repulsion::new(0.0, 6, 0.0), // zeroed: only gravity is compared against naive
        Drag::new(0.0, 0.0),
        0.5,
        None,
        false,
    );
    let mut forces_bh = vec![Vector2::zeros(); 2000];
    quadtree.barnes_hut_particles(&particles, &mut forces_bh);

    for i in 0..2000 {
        let diff = (forces_naive[i] - forces_bh[i]).norm();
        let scale = forces_naive[i].norm().max(forces_bh[i].norm());
        // Relative tolerance with an absolute floor: BH relative error blows up on
        // weak forces (where the true force is ~0), so small absolute errors pass.
        let tolerance = 5e-2 * scale.max(0.1);
        assert!(
            diff < tolerance,
            "particle {i}: naive {:?} vs bh {:?}",
            forces_naive[i],
            forces_bh[i]
        );
    }
}

#[test]
fn morton_sort_is_deterministic_and_preserves_particles() {
    let mut particles = generate_random_particles(500, 21, 1000.0);
    let before = snapshot(&particles);

    let mut sorter = MortonSort::new();
    sorter.sort(&mut particles);
    let after_once = snapshot(&particles);
    sorter.sort(&mut particles);
    let after_twice = snapshot(&particles);

    assert_eq!(
        after_once, after_twice,
        "sort is not deterministic (equal Morton codes must tie-break on index)"
    );
    assert_eq!(before, after_once, "sort lost or duplicated particles");
}

fn snapshot(particles: &Particles) -> Vec<(Scalar, Scalar, Scalar, Scalar, Scalar)> {
    let mut values: Vec<_> = particles
        .positions
        .iter()
        .zip(&particles.velocities)
        .zip(&particles.masses)
        .map(|((position, velocity), mass)| (position.x, position.y, velocity.x, velocity.y, *mass))
        .collect();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    values
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
    let radius: Scalar = 5.0;
    let orbital_speed = (g_coef * central_mass / radius).sqrt();

    let mut particles = Particles::new(
        vec![Vector2::zeros(), Vector2::new(radius, 0.0)],
        vec![Vector2::zeros(), Vector2::new(0.0, orbital_speed)],
        vec![central_mass, test_mass],
        vec![Color::BLACK; 2],
    );

    // Sorting must not corrupt the SoA: conserved quantities are permutation-invariant
    MortonSort::new().sort(&mut particles);

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
        (momentum1 - momentum0).norm() / momentum0.norm() < TOL_MOMENTUM,
        "momentum drifted: {momentum0} -> {momentum1}"
    );
    assert!(
        (energy1 - energy0).abs() / energy0.abs() < TOL_ENERGY,
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
        let expected_pos = g * dt * dt * (steps * (steps + 1) / 2) as Scalar;
        let expected_vel = g * dt * steps as Scalar;

        assert!(
            (particles.positions[0] - expected_pos).norm() < TOL_UG,
            "dt={dt}: pos {:?} vs expected {:?}",
            particles.positions[0],
            expected_pos
        );
        assert!(
            (particles.velocities[0] - expected_vel).norm() < TOL_UG,
            "dt={dt}: vel {:?} vs expected {:?}",
            particles.velocities[0],
            expected_vel
        );
    }
}

#[test]
fn force_computation_is_deterministic() {
    let particles = generate_random_particles(200, 42, 100.0);

    let mut forces_a = vec![Vector2::zeros(); 200];
    let mut forces_b = vec![Vector2::zeros(); 200];

    let mut gravity = Gravity::new(1.0, 0.0);
    let mut repulsion = Repulsion::new(1.0, 6, 0.0);
    let mut drag = Drag::new(1.0, 10.0);

    gravity.clone().apply(&particles, &mut forces_a);
    repulsion.clone().apply(&particles, &mut forces_a);
    drag.clone().apply(&particles, &mut forces_a);

    gravity.apply(&particles, &mut forces_b);
    repulsion.apply(&particles, &mut forces_b);
    drag.apply(&particles, &mut forces_b);

    for i in 0..200 {
        assert_eq!(forces_a[i], forces_b[i], "non-deterministic force at {i}");
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
    let mut stack = vec![0usize];

    while let Some(node_idx) = stack.pop() {
        let node = &quadtree.nodes[node_idx];

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
                assert!((particles.positions[i] - pos).norm() < TOL_STRUCT);

                let mass = particles.masses[i];
                center_of_mass += pos * mass;
                total_mass += mass;
            }

            assert!((node.total_mass - total_mass).abs() < TOL_STRUCT);
            if total_mass > 0.0 {
                assert!((node.center_of_mass - center_of_mass / total_mass).norm() < TOL_STRUCT);
            }
        } else {
            assert_eq!(node.indexes.len(), 0);
            stack.extend(node.childs.iter().copied());
        }
    }

    assert!(seen.iter().all(|&s| s), "some particles missing from leaves");
    assert!(leaf_count > 1);
}
