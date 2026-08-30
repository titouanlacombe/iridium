# Opti Campaign

Track optimization work. Baseline first, one change at a time, measure with `cargo bench`.

## Environment

- CPU: AMD Ryzen 5 5600X (6 cores / 12 threads)
- Threads (rayon default): 12
- Build: `cargo bench` (release, opt-level 3, debug symbols)
- NOTE: machine runs `powersave` governor + background load (Brave, load ~6) -> bench noise ~+-10%, treat small diffs as inconclusive

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
| 2 | `.cargo/config.toml`: `target-cpu=native` + lto/codegen-units=1 | H | L | L | done (see log) |
| 3 | Reuse per-thread scratch buffer (barnes-hut stack) | H | L | L | done (see log) |
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
| 2026-08-30 | #2 target-cpu=native + lto + codegen-units=1 | insertion 85.3, re-insertion 74.3, naive 11.6ms, bh 9.6ms, full_step 341-418µs | insertion 94.9, re-insertion 84.9, naive 11.8ms, bh 10.3ms, full_step 367µs | flat (within noise) |
| 2026-08-30 | #3 per-thread scratch stack in barnes_hut (for_each_init) | bh 10.3-11.3ms (run-to-run) | bh 10.6ms | flat (within noise) |
| 2026-08-30 | #3 v3 (final): per-thread stack via for_each_init, `<'a>` unified in helper | bh 10.3-11.3ms (run-to-run) | bh 9.3ms | low end of noise range, inconclusive |

### #3 verdict: final, flat (within noise)

Per-thread stack (one `Vec` per rayon thread, `clear()` per particle, capacity hint from `get_infos`). Chosen approach: `for_each_init`. Alternatives considered and rejected: `stack` field (self-referential -> needs lifetime param on QuadTree), recursion (no stack, no lifetime), arena + `Vec<usize>` (task #10 refactor). Allocator cost not visible at 3000 particles; revisit with a 100k-particle barnes_hut bench before judging.

### #2 verdict: no measurable gain

AVX2 confirmed in binary (110 ymm instr), yet all benches flat within +-10% noise -> the hot loops are **memory-bandwidth-bound**, not compute-bound. AVX2 doubles lanes but doesn't cut traffic.

Implication: SIMD compute micro-opts (TODO SIMD section) have limited headroom; the primary lever is **reducing memory traffic** (f32, fewer passes over buffers, scratch reuse). Revisit SIMD only after traffic work.
