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

## Development

```sh
RUST_BACKTRACE=1 RUST_LOG=info cargo run --release
```
