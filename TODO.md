multithreading {
	Create separate event handler struct (share Arc of sfml window?)
	get rid of sim2screen and screen2sim (events should be converted to sim coordinates by the renderer, coordinate system module)
	https://stackoverflow.com/questions/18860243/sdl-pollevent-vs-sdl-waitevent/18862404#18862404

	fix false random (pattern repeats each click)
	max fps sleep is should be handled when sending the draw command (less accurate but main thread wont block at weird places)

	multithread systems (barely worth, don't bother in some cases, test if faster with prints)
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
