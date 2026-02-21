# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A 3D space invaders game built with Bevy 0.18 in Rust (2024 edition). Features a player ship, alien invader grid, shooting mechanics, collision detection, and scoring.

## Build Commands

- **Build:** `cargo build`
- **Run:** `cargo run`
- **Test:** `cargo test`
- **Run single test:** `cargo test <test_name>`
- **Check (fast compile check):** `cargo check`
- **Lint:** `cargo clippy`
- **Format:** `cargo fmt`

## Architecture

Single-binary Rust project using Bevy's plugin architecture. Each gameplay system is a separate module/plugin:

```
src/
  main.rs          — App entry point, camera/lighting setup, plugin registration
  components.rs    — Shared marker components (Player, Enemy, Bullet), constants, resources
  player.rs        — PlayerPlugin: ship spawning, A/D movement, Space to shoot
  enemy.rs         — EnemyPlugin: 5×4 grid spawning, side-to-side + step-down movement, random shooting
  bullet.rs        — BulletPlugin: velocity-based movement, out-of-bounds cleanup
  collision.rs     — CollisionPlugin: distance-based bullet↔enemy and bullet↔player checks, game over
  scoreboard.rs    — ScoreboardPlugin: score UI text overlay, game over display
```

## Dependencies

- `bevy` 0.18 (with `3d` feature) — game engine
- `rand` 0.8 — random enemy shooting

## Game Controls

- **A / Left Arrow** — move player left
- **D / Right Arrow** — move player right
- **Space** — shoot
