opti barnes hut {
	bench & opti
	look into Fast Multipole Method (FMM) as replacement (O(n) vs O(n log n), accuracy via expansion order):
	- ferreus_bbfmm crate (kernel-independent, supports 2D quadtree, parallel)
	- kifmm (3D Laplace/Helmholtz, reference-grade)
	→ tried ferreus_bbfmm 0.2.0 + git main (0.3.0): 2D far-field y-gradient is broken
	  (values & x-gradients exact, y-gradients garbage ~1/cell-size, axis-intrinsic, all
	  compression/tree options, reproduced with single far source + 1/r kernel; -r kernel is fine)
	  min repro: target(100,100) src(500,600) → gy expected +1.9e-6 got -1.2e-3.
	  workaround: central-diff on exact potential values (4 extra evals, ~1e-4..1e-7 accuracy);
	  reopen once upstream fixes gradients (github.com/graphic-goose/ferreus_rbf_rs)
}

skip arrow/polars (dataframe libs add boxing/indirection, no SIMD gain over autovectorized loops) — if anything:
- tried glam instead of nalgebra (SIMD-accelerated, lighter): reverted — Vec2/DVec2 are not scalar-generic, would lose the Vector2<Scalar> plumbing used by the f32/f64 features; benches showed no real gain anyway
- or std::simd for the force kernels

github, logo & readme

finish QT {
	bench & opti
}

use izip?
change particles to Rc (remove simulation data altogether, only use systems)
implement particles lifetimes (new buffer) ?
sim events can control window (close, resize, etc)???
seal masses/inv_masses (private + accessors + push_mass/extend_masses) so the 1/m invariant is compiler-enforced; factory needs temp vec

iridium big facade to make it easy to use
more integrators option (euler, verlet, etc) & test biggest dt possible?
benchmark & optimize sim?

window set_icon
fullscreen key toggle
window.vsync

custom shading (geometry shader) {
	fix flipped y axis (cleanely) (need custom opengl?)
}
benchmark & optimize render?
