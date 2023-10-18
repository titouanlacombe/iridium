# Iridium

An easy to use and fast 2D particle simulator.

## Setup for development

### Dependencies

```sh
curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh
sudo apt install -y build-essential git python3 cmake sfml-dev
```

Use cargo to build and run the project.

```sh
cargo run
```

### Benchmarking

```sh
cargo bench
```

### Profiling

Setup:
```sh
sudo apt install -y perf
```

Run:
```sh
cargo build --release
perf record -g ./target/release/iridium
perf script > profile.linux-perf.txt
```

See result with [Speedscope](https://www.speedscope.app/).

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

## Development

```sh
RUST_BACKTRACE=full RUST_LOG=info cargo run --release
```
