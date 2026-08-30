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
| 4 | `1/mass` multiply instead of divide | M | L | L | done (see log) |
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

### #3 verdict: ~8% at 100k, borderline vs noise

100k-particle comparison (same bench file, base commit `2ff6262` = per-particle stack vs `2675a5b` = per-thread):

| bench (100k) | per-particle | per-thread | diff |
|---|---|---|---|
| insertion (identical code, = noise floor) | 6.85 ms | 6.39 ms | ~7% |
| re-insertion (identical code) | 5.57 ms | 5.47 ms | ~2% |
| barnes_hut | 458.9 ms | 421.9 ms | ~8% |

The insertion delta (~7%) on *identical* code measures run-to-run noise. barnes_hut at ~8% is the same magnitude -> real effect likely 0-8%, smaller than hoped (small allocs are cheap on glibc tcache; tree walk + memory traffic dominate). Keep the per-thread stack (removes 100k allocs/frame, no cost); don't expect it to show up visually.

Also: barnes_hut at 100k = 422 ms/frame (naive would be ~12.7 s) -> still far from realtime for large counts; that's the batched traversal / FMM headroom.

### #4 verdict: inv_masses in Particles, done

Tried 3 shapes:
1. per-frame reciprocal buffer in Physics (reciprocal pass + mul pass): mass_divide 106 µs vs 58-73 divide -> **~50% slower** (extra pass costs memory traffic; confirms memory-bound).
2. `inv_masses` stored in Particles SoA, computed once at creation (`Particles::new`, `swap_remove`, `clear`, `reserve_exact`, `shrink_to_fit`, `copy_from_indexes`, `GeneratorFactory::create`): mass_divide 56.3 µs, full_step 370 µs -> small win at best (within noise), zero divisions per frame.

Caveat: `masses` is still `pub` — a direct write to `particles.masses[i]` would desync `inv_masses` (documented on the field). If it ever becomes a problem, seal via accessors.

### #2 verdict: no measurable gain

AVX2 confirmed in binary (110 ymm instr), yet all benches flat within +-10% noise -> the hot loops are **memory-bandwidth-bound**, not compute-bound. AVX2 doubles lanes but doesn't cut traffic.

Implication: SIMD compute micro-opts (TODO SIMD section) have limited headroom; the primary lever is **reducing memory traffic** (f32, fewer passes over buffers, scratch reuse). Revisit SIMD only after traffic work.
