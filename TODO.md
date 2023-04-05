Simulation events (time of execution, lambda taking simulation as argument) => store sorted list of events
class particles => store buffers for pos, vel, mass, color, forces, etc interact with them like there were individual particles {
	multithreaded iterator (hold thread pool & stack size)
}
forces are updatables
integrator is updatable (done at the end of the step)

builders object to make simulation creation easier
create config for integrator option (euler, verlet, etc)
is it posible to implement particles lifetimes ?
create yaml config file?

benchmark & optimize

add quadtree (readme.md)
add coulomb, friction, gravity

add colors
custom shading (geometry shader)

benchmark & optimize

notes:
for sim benchmark disable render
