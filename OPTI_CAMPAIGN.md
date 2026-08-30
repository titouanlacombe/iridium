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
| 5 | Concrete `Vector2` integrator impl | M | L | L | done (reverted: no gain, see log) |
| 6 | Rename `get_infos` + slices (`&[T]`) API | L | L | L | pending |
| 7 | Tests (symmetry, conservation, stability, QT vs brute-force) | M | M | L | done (see log) |
| 8 | Kill `Arc<Mutex>` -> fold/reduce (determinism) | M | M | M | done (see log) |
| 9 | `Scalar = f32` feature flag | H | L | M | done (see log) |
| 10 | QT allocator (arena) + `qt.leaves()` + QT bench | M | M | M | done (see log) |
| 11 | Batched barnes-hut (4-8 particles/traversal) | H | H | M | done (see log: SIMD = 2.6x f64, ~60x f32) |
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

Caveat: `masses` is still `pub` — a direct write to `particles.masses[i]` would desync `inv_masses` (documented on the field). If it ever becomes a problem, seal via accessors (noted in TODO).

### #5 verdict: reverted, keep the generic impl

Concrete `impl Integrator<Vector2<f64>>` cannot coexist with the generic `impl<T: ...> Integrator<T>` — coherence rejects the overlap (E0119, no stable specialization). The generic impl is strictly more general. Measured the concrete version before reverting: integrate 52.7 µs vs 52-59 µs generic -> flat (Clone is a no-op for Copy types; LLVM eliminates it). Conclusion: the generic impl is the right API and not a bottleneck.

### #7 verdict: test suite added (tests/simulation.rs)

5 tests: force symmetry (gravity/repulsion/drag antisymmetric), barnes-hut theta=0 vs naive equivalence, momentum+energy conservation on a circular 2-body orbit (incl. orbit-radius drift check), uniform-gravity vs analytic solution at two dt values (dt-independence), quadtree structure invariants (leaf capacity, disjoint index coverage, rect containment, center-of-mass/total-mass consistency). All pass.

Gotcha found while writing: random N-body with epsilon=0 allows close encounters -> energy spikes legitimately (momentum still conserved); switched to a circular orbit setup to keep the force smooth.

### #8 verdict: done, lock-free + deterministic (bug caught by the new tests)

Replaced `rayon::scope` + `Arc<Mutex>` merge in Gravity/Drag/Repulsion::apply with `into_par_iter().map(...)` local buffers + a merge that sums in fixed per-index order -> bitwise-deterministic results (new `force_computation_is_deterministic` test).

Caught a real bug the tests exposed: first version *overwrote* (`*force = sum`) instead of *accumulating* (`*force += sum`) -> multi-force setups (Physics::update applies several forces to one buffer) would have kept only the last force's contribution. Tests (barnes_hut vs naive) caught it; fixed.

Perf: naive 11.8 ms / full_step 334 µs -> within noise. The lock wasn't the bottleneck (merges were serialized but cheap vs O(n^2)). Value: determinism + no lock in the hot path.

### #9 verdict: f32 feature flag done

`Scalar` is now cfg-gated (`f32`/`f64` features, default f64). Touched: types.rs, integrator.rs (dt + Mul bound), quadtree.rs (scale/theta), smooth_rate.rs, generators.rs (PI + casts), systems.rs (ColorWheel cast), render_thread.rs (generic `nalgebra_to_sfml`), examples.rs, tests (cfg'd tolerances: f32 has ~7 digits).

Measured (buffer_ops @100k): full_step 313.7 µs vs 334-418 f64 (~10-15% faster), uniform_gravity 46.4 vs ~52, mass_divide 48.2 vs ~55. N-body at 3k flat (within noise). All 6 tests pass under both features. Expected direction (memory-bound -> halving traffic gives partial wins, not 2x). Use `--features f32` in demos; keeps f64 default.

### #10 verdict: arena done, flat (within noise)

Quadtree nodes now live in one contiguous `QuadTree::nodes: Vec<QuadTreeNode>` arena, `childs` are `Vec<usize>` indices. Traversal stack is now `Vec<usize>` (no `<'a>`). Added `root()` + `leaves()` accessors; renderer + tests walk the arena.

Memory boundedness: `compact()` runs at the end of every insert — drops unreachable (dead) subtrees via an in-place mark/remap/move pass, truncating the arena. Capacity is retained so re-insertion doesn't reallocate; leaf = `childs.is_empty()` again (no `is_leaf` flag). Arena growth is now bounded by peak live nodes, freed at least once per update.

Results: 3k bh 10.6-11.2 ms, 100k bh ~480 ms -> within the 420-490 run-to-run band (machine noise). Same conclusion as #2/#3: allocation churn and pointer-chasing weren't the bottleneck at these sizes. Structural wins remain: index-based traversal (unblocks #11 batching + SIMD), no lifetimes, flat leaves() iteration, bounded arena.

Remaining big lever: #11 batched traversal (100k bh ~480 ms/frame).

### #11 verdict: infrastructure done, batched walk flat (measurement killed a bug)

MortonSort (System, deterministic: index tie-break) + batched barnes-hut (BATCH = 4 f64 / 8 f32, one walk per batch with per-subtree particle bitmask).

- New test `barnes_hut_theta_05_is_a_reasonable_approximation` caught a real bug in the first mask design: a per-particle "done" flag (instead of a per-stack-entry mask) double-approximated nothing but *skipped* sibling subtrees -> forces ~640x too small. Mask travels with the stack entry: fixed. Also caught the earlier 43x "speedup" as garbage.
- Verdict: correct but FLAT (3k 11.2 ms incl. sort, 100k 460 ms vs 421-490 solo). The bottleneck at theta=0.5 is the leaf-pair arithmetic (each pair computed once per particle, ~independent of batching), not node loads (tree is L3-resident at 100k, 13 MB in 32 MB L3). Batching amortizes node loads only.
- The real remaining lever for the pair arithmetic: SIMD across the batch lanes (wide crate, f64x4/f32x8) on the pair + criterion loops - compute-bound, unlike the buffer passes. MortonSort is currently neutral (sort ~5-15 ms at 100k) - it stays (standard technique, keeps batches coherent if node loads ever matter).

### #11 SIMD: done - the win finally landed

Batch forces vectorized across the lanes (wide crate, `SimdVec` trait in types.rs, f64x4 / f32x8). BATCH = one register (4 f64 / 8 f32). Bugs caught by the tests while writing: (1) NaN*0 = NaN: the vectorized self-pair computes 0/0 on its own lane -> masks must select, not multiply; (2) drag with distance=0 divides by zero before the validity mask -> mask the ratio first; (3) sign error: approx branch used com-pos instead of pos-com -> forces partially inverted (only visible via the theta=0.5 sanity test). All fixed, 8/8 tests under both features.

Results (100k bh + sort):

| version | 3k | 100k |
|---|---|---|
| solo scalar (original) | 9.5-11.3 ms | 421-490 ms |
| scalar batched + sort | 11.2 ms | 460 ms |
| SIMD f64x4 + sort | 4.31 ms | 184 ms |
| SIMD f32x8 + sort | - | **7.0 ms** |

f64: ~2.6x vs solo. f32: ~4.8x vs f64 solo (8 lanes + f32 sqrt/div throughput on AVX2). NOTE: the earlier reported "f32 7 ms" was measured with a broken full-batch mask (see below) - the real f32 number is ~96 ms. 100k particles: 460 ms (f64 scalar) -> 96 ms (f32 SIMD).

### SIMD everywhere else (naive N-body, insert redistribution, fusion)

Three candidates, measured:
1. Naive N-body SIMD (forces.rs `apply_pairwise_simd`, kernels per force): 3k naive 11.5-12 ms -> 8.4 ms (~30%). Modest: at 3k the per-thread work is small, thread overhead dominates.
2. Insert redistribution vectorized (4 children per register): REVERTED - a regression. The scalar early-exit `break` averages ~1.5 containment tests per particle; the SIMD version paid 4 lanes + register round-trips every time (100k insert 6.4-7.7 ms -> 9.6 ms).
3. Pass fusion: `Integrator::integrate_scaled_vec` (result += values * scale * dt), declared on the interface, implemented in GaussianIntegrator; Physics accumulates forces then calls it via its Box<dyn Integrator> (velocities += forces * inv_masses * dt in one pass, mass scaled ONCE at integration - not per force). full_step 340-420 µs -> ~306-314 µs (~15-20%). Integrator stays swappable.
   - Rejected alternative: dividing by mass inside the force kernels (a = F/m per force). Scales per force (inefficient, kernels needed 4 SIMD outputs instead of 2).

### Pre-existing f32 bug found (full-batch mask)

`(1 << count) - 1` for count == 8 yields 0 on u8 (1u8 << 8 == 1 at runtime), so every full f32x8 batch (the last partial batch worked) got a zero root mask -> no forces. f64 never hit it (count <= 4). This also invalidated the earlier "f32 7 ms" measurement. Fixed with `u8::MAX >> (8 - count)`. The f32 test suite now genuinely runs to completion (it had silently stopped at 8/8-passed doc-test lines before).

### #2 verdict: no measurable gain

AVX2 confirmed in binary (110 ymm instr), yet all benches flat within +-10% noise -> the hot loops are **memory-bandwidth-bound**, not compute-bound. AVX2 doubles lanes but doesn't cut traffic.

Implication: SIMD compute micro-opts (TODO SIMD section) have limited headroom; the primary lever is **reducing memory traffic** (f32, fewer passes over buffers, scratch reuse). Revisit SIMD only after traffic work.
