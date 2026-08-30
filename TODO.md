opti barnes hut {
	bench & opti
	reuse per-thread scratch buffer for the traversal stack (allocated per particle right now, quadtree.rs)
	look into Fast Multipole Method (FMM) as replacement (O(n) vs O(n log n), accuracy via expansion order):
	- ferreus_bbfmm crate (kernel-independent, supports 2D quadtree, parallel)
	- kifmm (3D Laplace/Helmholtz, reference-grade)
}

force compute {
	kill Arc<Mutex> accumulation in N-body forces (forces.rs): fold/reduce over partitions or final parallel sum of local vecs
	=> also makes results deterministic (accumulation order is currently racy)
	forces_buffer /= mass -> multiply by precomputed 1/mass
}

skip arrow/polars (dataframe libs add boxing/indirection, no SIMD gain over autovectorized loops) — if anything:
- use glam instead of nalgebra (SIMD-accelerated, lighter)
- or std::simd for the force kernels

SIMD acceleration of buffer ops {
	easiest win first: .cargo/config.toml with "-C target-cpu=native" + lto/codegen-units=1, bench it
	try Scalar = f32 as feature flag (types.rs): halves memory traffic, doubles SIMD lanes, bench precision [done, ~10-15% on buffer ops]
	concrete Vector2 integrator impl (generic + Clone inhibits autovectorization)
	explicit SIMD (wide crate, f64x4) if needed: split positions x/y into separate arrays (true SoA)
	batched barnes-hut: walk tree once per 4-8 particles, forces lane-parallel
}

tests {
	force symmetry (F_ij == -F_ji)
	momentum/energy conservation over N steps
	integrator stability at large dt
	quadtree/barnes-hut vs brute-force equivalence on small N
}

github, logo & readme

finish QT {
	create a function to return list of refs instead of iter (qt.leaves())
	bench & opti
	allocator (custom vec allocator using swap remove?)
}

use izip?
change particles to Rc (remove simulation data altogether, only use systems)
implement particles lifetimes (new buffer) ?
sim events can control window (close, resize, etc)???
seal masses/inv_masses (private + accessors + push_mass/extend_masses) so the 1/m invariant is compiler-enforced; factory needs temp vec

iridium big facade to make it easy to use
more integrators option (euler, verlet, etc) & test biggest dt possible?
benchmark & optimize sim?
benches: use constant dt (demos are interactive -> variable dt ok, just clamp max dt to avoid instability)
add criterion benches for buffer ops (uniform forces, integrator, mass divide) to track SIMD work
use slices (&[T]) instead of &Vec<T> in integrator/particles APIs

window set_icon
fullscreen key toggle
window.vsync

custom shading (geometry shader) {
	fix flipped y axis (cleanely) (need custom opengl?)
}
benchmark & optimize render?
