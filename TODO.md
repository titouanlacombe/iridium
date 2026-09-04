opti barnes hut {
	bench & opti
	look into Fast Multipole Method (FMM) as replacement (O(n) vs O(n log n), accuracy via expansion order):
	- ferreus_bbfmm crate (kernel-independent, supports 2D quadtree, parallel)
	- kifmm (3D Laplace/Helmholtz, reference-grade)
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
