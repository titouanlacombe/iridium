# Iridium

An easy-to-use and fast 2D particle simulator.

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

Setup:
```sh
sudo apt install -y perf
```

Run:
```sh
cargo build --profile bench
perf record --call-graph dwarf ./target/release/iridium
perf script -F +pid > iridium.perf 
```

See the result with the Firefox profiler:
https://profiler.firefox.com/

## Features

Easy to use: clean but powerful API

Fast: written in Rust

Forces:
- Gravity (uniform) 
- Drag (uniform)
- Gravity (particle to particle) (not yet implemented)
- Coulomb (particle to particle) (not yet implemented)
- Drag (particle to particle) (not yet implemented)

## Development

```sh
RUST_BACKTRACE=full RUST_LOG=info cargo run --release
```
