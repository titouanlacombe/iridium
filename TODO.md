window event handler in renderer
<!-- type WindowEventHandler = Box<dyn FnMut(&mut IridiumRenderer, Event)>; -->

class particles => store buffers for pos, vel, mass, color, forces, etc interact with them like there were individual particles
forces are systems
integrator is system (warning, need to be at the end of the update list)
particles multithreaded iterator (hold thread pool & stack size)
fix Simulation events: taking more than just particles (hopefully also systems)

builders object to make simulation creation easier
more integrators option (euler, verlet, etc) & test smallest dt possible?
is it posible to implement particles lifetimes ?
create yaml config file?
add particle color
benchmark & optimize

add quadtree (readme.md)
add coulomb, friction, gravity
benchmark & optimize

custom shading (geometry shader)
benchmark & optimize gpu?

notes:
for sim benchmark disable render
