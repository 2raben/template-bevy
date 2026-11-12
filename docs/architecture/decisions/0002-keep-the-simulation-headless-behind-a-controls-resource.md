---
tags: [adr]
status: accepted
date: 2026-11-12
sources:
  - src/main.rs
  - src/simulation.rs
  - src/input.rs
  - src/visuals.rs
  - src/tests.rs
---

# 0002 — Keep the simulation headless behind a `Controls` resource

## Context

Game rules are the code most worth testing and the code most often tangled with
devices and rendering. If a gameplay system reads the keyboard or spawns sprites,
tests need a window, a GPU, or fake devices, and frame-rate dependent input bugs
appear where fixed-step systems miss presses.

Alternatives considered:

- **Read devices directly in gameplay systems.** Least code, but untestable
  headless, and a press that lands on a frame with no fixed step is lost.
- **An input-action crate** such as `leafwing-input-manager`. Useful for rebinding
  and many actions, but more than a starter needs; it can be adopted later behind
  the same boundary.
- **A plain `Controls` resource** written by one input plugin and read by fixed-step
  systems.

## Decision

[main.rs](../../../src/main.rs) composes two layers of plugins.

- **Simulation** ([simulation.rs](../../../src/simulation.rs)): level, player,
  coins, and run rules. It adds no rendering, windowing, or device input and runs
  under `MinimalPlugins`.
- **Presentation**: [input.rs](../../../src/input.rs) samples devices in
  `PreUpdate` after `InputSystems` and writes `Controls`;
  [visuals.rs](../../../src/visuals.rs), `camera.rs`, and `hud.rs` read simulation
  components and resources. Visuals add sprites to simulation entities when they
  appear (`Added<Player>`, `Added<Coin>`).

`Controls` holds held values (`move_x`, `jump_held`) and latched presses
(`jump_pressed`, `restart`). The input layer only sets latched presses; the
fixed-step system that consumes one clears it, so each press acts exactly once
whether a frame runs zero, one, or several fixed steps.

## Consequences

- Tests drive the game by writing `Controls`, exactly as the input layer does.
- Adding a device or rebinding touches only `input.rs`.
- Presentation can change freely without affecting the tests, which also means
  the tests do not catch visual or input-mapping bugs; check those in the game.
- New simulation state must be readable as components or resources, not hidden
  in presentation code.

## The tell

- `rg -n "ButtonInput|Gamepad|KeyCode" src` matches only `src/input.rs`.
- `rg -n "Sprite|Camera2d|Text::|Node \{" src/simulation.rs src/player.rs src/run.rs src/level.rs`
  finds nothing.
- `cargo test --locked` builds the simulation without `DefaultPlugins`; a test that
  starts needing a window means presentation leaked into the simulation.
