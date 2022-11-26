# TODO

create vfx pipeline

particle add temperature (friction => heat)
particle add pressure (only for rendering?)

Simulation builder object for easy api
emitter & consumer builder objects

add quadtree?
add hard forces (coulomb, friction, gravity)

notes:
for sim benchmark disable render

friction generates heat => black body => color

// How to handle cache ? particle pos needed in big struct & in quadtree?
quadtree.update_pos() {
	let moved = []

	if self.is_leaf() {
		foreach particle in self.particles {
			if not self.contains(particle) {
				moved.push(particle);
			}
		}
	}
	else {
		foreach child in self.children {
			moved.concat(child.update());
		}

		// Process moved set
		foreach particle in moved {
			if self.contains(particle) {
				moved.swap_remove(particle);
				self.assign(particle);
			}
		}
	}

	return moved;
}

quadtree.update_struct() {
	let particles;
	if self.is_leaf() {
		particles = self.particles.len();
		if particles > self.max_particles {
			self.split();
		}
	}
	else {
		particles = 0;
		foreach child in self.children {
			particles += child.update_struct();
		}
		if particles < self.max_particles {
			self.merge();
		}
	}
	return particles;
}
