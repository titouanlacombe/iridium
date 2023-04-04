simulation runner interface (run simulation with delta times) in the future will be his own thread
refactor emitters sub 1 rate
class particles => store buffers for pos, vel, mass, color, etc interact with them like there were individual particles
create yaml config file?
create config for integrator option (euler, verlet, etc)
UI fps limiter option
multithreaded particle update (custom foreach with stack size & thread pool)

Simulation events (time of execution, lambda taking simulation as argument) => store sorted list of events
Simulation builder object
emitter & consumer builder objects

is it posible to implement particles lifetimes ?
benchmark & optimize

add quadtree (readme.md)
add coulomb, friction, gravity

custom shading (geometry shader)

notes:
for sim benchmark disable render
