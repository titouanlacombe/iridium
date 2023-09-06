multithreading {
	Mutex to RwLock (optimize read access) or alternative to arc for single thread
	
	https://stackoverflow.com/questions/18860243/sdl-pollevent-vs-sdl-waitevent/18862404#18862404

	refactor commands: self.command<T: Command>(Command::Draw::new()) -> Command::Draw::Result

	fix false random (pattern repeats each click)

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
