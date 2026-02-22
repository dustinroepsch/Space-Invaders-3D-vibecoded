# Invaders 3D

A 3D space invaders game built with [Bevy 0.18](https://bevyengine.org/) in Rust. Features voxel-style 3D graphics, multi-wave progression, destructible barriers, a mystery ship, procedural audio, and a CRT post-process shader.

**[▶ Play in your browser](https://dustinroepsch.github.io/Space-Invaders-3D-vibecoded/)**

## Controls

| Key | Action |
|-----|--------|
| `A` / `←` | Move left |
| `D` / `→` | Move right |
| `Space` | Shoot |

## Building

### Native

```bash
cargo run
```

### Web (WASM)

```bash
trunk serve       # dev server at localhost:8080
trunk build       # production build → dist/
```

Requires [trunk](https://trunkrs.dev/) and the `wasm32-unknown-unknown` target:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```
