# Iridium

An easy to use and fast particle simulator.

## Setup for development

```sh
sudo apt-get install build-essential git python cmake libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev libsdl2-mixer-dev libsdl2-gfx-dev libvulkan-dev vulkan-utils
```

Use cargo to build and run the project.

```sh
cargo run
```

## Features

Easy to use: clean but powerful API
Fast: written in Rust
Forces:
- Gravity (n-body)
- Gravity (uniform field) 
- Coulomb
- Drag
