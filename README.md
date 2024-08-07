# Iridium

An easy to use and fast 2D particle simulator.

## Features

Easy to use: clean but powerful API

Fast: written in Rust

Forces:
- Gravity (uniform) 
- Gravity (particle to particle)
- Coulomb (particle to particle)
- Drag (uniform)
- Drag (particle to particle)

Quadtree: fast collision detection and force computation

## Setup for development

### Dependencies

```sh
curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh
sudo apt install -y build-essential git python3 cmake sfml-dev
```

## Build and run

```sh
RUST_BACKTRACE=full RUST_LOG=info cargo run --release
```

### Benchmarking

```sh
cargo bench
```

### Profiling

This project is using [Tracy](https://github.com/wolfpld/tracy) to profile the build.

Build the Tracy server, run it.

Then run the application, it should connect to the tracy server.
