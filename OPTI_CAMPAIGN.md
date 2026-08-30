# Opti Campaign

Track optimization work. Baseline first, one change at a time, measure with `cargo bench`.

## Environment

- CPU: AMD Ryzen 5 5600X (6 cores / 12 threads)
- Threads (rayon default): 12
- Build: `cargo bench` (release, opt-level 3, debug symbols)

## Baseline (2026-08-30)

No `.cargo/config.toml` tuning, deps updated (rand 0.10).

### quadtree (3000 particles)

| bench | time |
|---|---|
| insertion | 85.3 µs |
| re-insertion | 74.3 µs |
| naive | 11.8 ms |
| barnes_hut | 9.6 ms |

### buffer_ops (100k particles, dt = 1/120)

| bench | time |
|---|---|
| uniform_gravity | 50 µs |
| uniform_drag | 51 µs |
| mass_divide | 58 µs |
| integrate | 52 µs |
| full_step | 341 µs |

## Task matrix

| # | Task | Impact | Effort | Risk | Status |
|---|---|---|---|---|---|
| 1 | Criterion benches (buffer ops) + constant dt | M | L | L | done |
| 2 | `.cargo/config.toml`: `target-cpu=native` + lto/codegen-units=1 | H | L | L | pending |
| 3 | Reuse per-thread scratch buffer (barnes-hut stack) | H | L | L | pending |
| 4 | `1/mass` multiply instead of divide | M | L | L | pending |
| 5 | Concrete `Vector2` integrator impl | M | L | L | pending |
| 6 | Rename `get_infos` + slices (`&[T]`) API | L | L | L | pending |
| 7 | Tests (symmetry, conservation, stability, QT vs brute-force) | M | M | L | pending |
| 8 | Kill `Arc<Mutex>` -> fold/reduce (determinism) | M | M | M | pending |
| 9 | `Scalar = f32` feature flag | H | L | M | pending |
| 10 | QT allocator (arena) + `qt.leaves()` + QT bench | M | M | M | pending |
| 11 | Batched barnes-hut (4-8 particles/traversal) | H | H | M | pending |
| 12 | FMM (ferreus_bbfmm/kifmm research) | H | VH | H | pending |
| 13 | Integrators (verlet...) + max-dt tests | M | M | L | pending |
| 14 | Facade + error handling (thiserror) | M | M | L | pending |

## Results log

| Date | Change | Before | After | Diff |
|---|---|---|---|---|
| 2026-08-30 | baseline | see above | - | - |
