multithreading {
	test C++ version of test (parallel iterate over buffer)
	
	multithreading simulation (learned that only worth when there is a lot of computation for small amount of data)
	render thread (need to duplicate position & color buffers?)
}

trait spatial partitioning
add quadtree (readme.md)
create boids system
add coulomb, friction, gravity
benchmark & optimize

iridium big facade to make it easy to use
is it posible to implement particles lifetimes ?
more integrators option (euler, verlet, etc) & test smallest dt possible?
custom shading (geometry shader)
benchmark & optimize gpu?

notes:
for sim benchmark disable render
