finish QT {
	allocator
	draw qt (Rc...)
	bench & opti
	implement coulomb, friction with quadtree
}

use izip?
change particles to Rc (remove simulation data altogether, only use systems)
implement particles lifetimes (new buffer) ?
sim events can control window (close, resize, etc)???

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
