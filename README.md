# Iridium

An easy to use and fast 2D particle simulator.

## Setup for development

### Dependencies

```sh
curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh
sudo apt install -y build-essential git python3 cmake libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev libsdl2-mixer-dev libsdl2-gfx-dev libvulkan-dev vulkan-tools
```

Use cargo to build and run the project.

```sh
cargo run
```

### Benchmarking

Setup:
```sh
sudo apt install -y perf
```

Run:
```sh
cargo build --release
perf record -g -F 999 ./target/release/iridium
perf script -F +pid > iridium.perf 
```

See result with firefox profiler:
https://profiler.firefox.com/

## Features

Easy to use: clean but powerful API

Fast: written in Rust

Forces:
- Gravity (uniform) 
- Gravity (particle to particle)
- Coulomb (particle to particle)
- Drag (uniform)
- Drag (particle to particle)
