# Iridium

An easy to use and fast particle simulator.

## Features

Easy to use: clean but powerful API
Fast: written in Rust
Forces:
- Gravity (n-body)
- Gravity (uniform field) 
- Coulomb
- Drag

## Data structures

### Particle

- position
- velocity
- mass

### Tap

- position
- radius
- rate
- p_factory

### Drain

- position
- radius
- max_rate

### QuadTree

- position
- width
- height
- child_0
- child_1
- child_2
- child_3

### ParticleCollection

- particles
- free_map

