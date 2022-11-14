# Iridium

An easy to use and fast 2D particle simulator.

## Setup for development

### Dependencies

TODO: use mold linker by upgrading to ubuntu 20.04

```sh
curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh
sudo apt install -y build-essential git python3 cmake lld
```

Use cargo to build and run the project.

```sh
cargo run
```

### Benchmarking

Setup:
```sh
sudo apt install -y valgrind kcachegrind
```

Run:
```sh
cargo build --release
valgrind --tool=callgrind ./target/release/iridium
kcachegrind <file generated by valgrind>
```

## Features

Easy to use: clean but powerful API
Fast: written in Rust
Forces:
- Gravity (uniform) 
- Gravity (particle to particle)
- Coulomb (particle to particle)
- Drag (uniform)
- Drag (particle to particle)
