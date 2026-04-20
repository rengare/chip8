# CHIP-8 Emulator

A CHIP-8 emulator written in Rust, targeting both desktop and web (via WebAssembly).

## Project Structure

- `chip8_core` — Core emulation library (CPU, opcodes, display, input)
- `desktop` — Desktop frontend
- `wasm` — WebAssembly frontend
- `web` — Web assets
- `xtask` — Build tooling

## Building

### Desktop

```sh
cargo build --release -p desktop
```

### WebAssembly

```sh
cargo xtask build-wasm
```

## Credits

Built following the tutorial:
**An Introduction to Chip-8 Emulation using the Rust Programming Language** by Austin Bricker (aquova)
- https://aquova.net/emudev/chip8/
- https://github.com/aquova/chip8-book
