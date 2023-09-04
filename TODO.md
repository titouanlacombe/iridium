multithreading {
	multithread systems (barely worth, don't bother in some cases, test if faster with prints)

	async draw call (simulate, wait draw thread/call, prepare draw call (create buffer directly on GPU? test if faster), async draw call, loop)
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
