# Architecture

This document describes the current implementation and conventions for extending
it. [AGENTS.md](../../AGENTS.md) is the agent working brief;
[CONTRIBUTING.md](../../CONTRIBUTING.md) covers shaping, review, and validation.
[ADRs](decisions/README.md) record significant architectural choices and rationale.

## Current structure and runtime

The project is one Rust binary crate implementing the
[starter platformer pitch](../pitches/starter-platformer/pitch--starter-platformer.md):
one level from a text map, a player who runs and jumps, coins, a goal flag, a
following camera, and a HUD. [Cargo.toml](../../Cargo.toml) uses Rust edition 2024
and Bevy 0.20 with default features disabled and the `2d`, `audio`, and `ui`
feature sets enabled. Bevy is the only dependency; movement is hand-written
([ADR 0001](decisions/0001-move-the-player-with-hand-written-kinematics.md)).
Dependencies have a higher development optimization level than the application.
[Cargo.lock](../../Cargo.lock) pins dependency resolution;
[.cargo/config.toml](../../.cargo/config.toml) configures Clang and LLD for x86_64
Linux, and [.mise/](../../.mise/config.toml) pins the toolchain and defines tasks
([ADR 0003](decisions/0003-run-project-tasks-with-mise.md)).

[src/main.rs](../../src/main.rs) composes the app from plugins in two layers
([ADR 0002](decisions/0002-keep-the-simulation-headless-behind-a-controls-resource.md)).

**Simulation**, which runs without a window and is used by the tests:

- [simulation.rs](../../src/simulation.rs): `SimulationPlugin` registers the
  resources, spawns the player and coins at startup, and orders the fixed-step
  systems in the `GameSystems` set.
- [level.rs](../../src/level.rs): the `Level` resource parsed from `STARTER_MAP`.
  `#` tiles are merged into horizontal runs of solid rectangles; `P`, `G`, and `o`
  mark the start, goal, and coins. The origin is the map's bottom-left corner and a
  tile is 32 world units.
- [player.rs](../../src/player.rs): the `Player` marker, `Position` (feet),
  `PreviousPosition`, `Velocity`, `Grounding`, the `Controls` input bridge,
  `PlayerTuning`, `step_player`, and the pure `move_and_collide`.
- [run.rs](../../src/run.rs): the `Run` resource (coins, time, falls, finished) and
  the rules for coins, the goal, falling out, restarting, and the clock.

**Presentation**, which needs a window:

- [input.rs](../../src/input.rs): keyboard and gamepad sampling into `Controls`, and
  Escape to quit. The only module that reads devices.
- [visuals.rs](../../src/visuals.rs): the palette, coloured sprites for the level,
  goal, coins, and player, render interpolation, and facing.
- [camera.rs](../../src/camera.rs): a `Camera2d` with a minimum 960 × 540 view that
  follows the player and stays inside the level.
- [hud.rs](../../src/hud.rs): the progress panel, controls reminder, and finish cue.

### Frame and fixed-step flow

1. `PreUpdate`, after Bevy's `InputSystems`: `sample_controls` writes held values
   and latches presses into `Controls`. After the goal only Restart is accepted.
2. `FixedUpdate` at 64 Hz, chained in `GameSystems`: `restart_run` consumes a
   restart; `step_player` consumes a jump press, steers, jumps, applies gravity, and
   collides; then `collect_coins`, `reach_goal`, `fall_out`, and `tick_clock`.
3. `Update`: visuals add sprites to newly spawned players and coins; the HUD
   refreshes when `Run` changes.
4. `PostUpdate`, before transform propagation: positions are interpolated into
   transforms, the player faces its direction of travel, and then the camera
   follows the player.

### Movement model

The player is a 24 × 40 box anchored at the centre of its feet. Horizontal speed
moves toward the input at a ground or air acceleration. A jump fires when a press
is buffered (0.12 s) and the player is grounded or within coyote time (0.1 s) of
leaving the ground. Releasing jump while rising caps the upward speed, giving a
short hop. Gravity is capped at a terminal fall speed. `move_and_collide` moves
along x, then y, clamping against overlapping solids; touching edges do not count
as overlap, so the player slides along the floor. A downward block marks the player
grounded. With the default tuning a held jump rises about three and a half tiles and a
running jump clears four.

## Code conventions and boundaries

Use Bevy entities and components for game objects, resources for shared state, and
systems for behavior. Keep application composition easy to locate in `main.rs`.
Keep simulation plugins free of rendering and device input so they keep running
headless; presentation plugins react to simulation components and resources.

Follow rustfmt and nearby Rust naming and import conventions. Prefer small,
purposeful systems with explicit data access. Make scheduling dependencies explicit
when behavior depends on order. Read device input only in `input.rs` and pass it to
fixed-step systems through `Controls`. Keep pure logic, such as map parsing and
collision, in plain functions that unit tests can call directly.

New dependencies, physics integration, simulation timing, and persisted formats can
create lasting constraints; explain such choices in an ADR. Grow modules into
folders (`src/player/mod.rs` and friends) or new plugins only when a feature needs
them.

## Validation

The [contribution guide](../../CONTRIBUTING.md#setup-and-validation) lists the
formatting, Clippy, build, test, and manual-run checks. There is no CI workflow.

- **Unit tests** beside each module cover map parsing and errors, `move_and_collide`
  against floors, walls, and ceilings, and camera bounds.
- **Simulation tests** in [src/tests.rs](../../src/tests.rs) step the real plugins
  headless, one fixed step per update. They cover landing, running and stopping,
  held and tapped jump heights, a running jump over a four-tile gap, no midair
  jumps, buffered jumps, walls, coins, the goal and clock, falling out, and restart.

These tests guard behavior, not feel. Play the level after any tuning change, and
check visuals and input by hand.

## Documentation boundaries

- [Game idea](../idea.md): direction and longer-term possibilities.
- [Pitches](../pitches/README.md): intended work, rough solutions, risks, and
  exclusions.
- Architecture documentation: current structure, behavior, and coding conventions.
- [ADRs](decisions/README.md): significant choices, alternatives, consequences, and
  checks that expose drift.

Update conventions with the code. Keep ideas clearly separate from implemented
behavior. Preserve decision history by amending or superseding an ADR when its
choice changes.
