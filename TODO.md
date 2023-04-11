class particles => store buffers for pos, vel, mass, color interact with them like there were individual particles
create physics system (forces are part of that system, use internal force buffer + integrator)
trait for renderers
particles multithreaded iterator (hold thread pool & cluster size)
fix Simulation events: taking more than just particles (hopefully also systems)

builders object to make simulation creation easier?
iridium big facade
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
