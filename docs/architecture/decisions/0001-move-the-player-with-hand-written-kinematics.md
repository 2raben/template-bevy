---
tags: [adr]
status: accepted
date: 2026-11-12
sources:
  - Cargo.toml
  - src/player.rs
  - src/simulation.rs
  - src/visuals.rs
  - src/tests.rs
  - ../../pitches/starter-platformer/pitch--starter-platformer.md
---

# 0001 — Move the player with hand-written kinematics on a fixed timestep

## Context

The [starter platformer pitch](../../pitches/starter-platformer/pitch--starter-platformer.md)
needs one box that runs, jumps, and lands on axis-aligned tile geometry, with
behaviour that tests can check exactly. The template should also show readers
where movement code lives rather than hide it inside an engine.

Alternatives considered:

- **A physics engine** such as `avian2d` or `bevy_rapier2d`. They handle rotation,
  slopes, and stacking, none of which the starter needs, and add compile time,
  version coupling to Bevy, and tuning that is harder to read. A project that needs
  rigid bodies can adopt one and supersede this record.
- **Variable-timestep movement in `Update`.** Simpler scheduling, but jump heights
  and collisions would depend on frame rate and tests could not step exactly.
- **Hand-written kinematics on Bevy's fixed timestep.** A few dozen lines, exact
  and frame-rate independent.

## Decision

The player is a kinematic box. [player.rs](../../../src/player.rs) keeps its feet
position, velocity, and grounding in components; `step_player` runs in
`FixedUpdate` at Bevy's default 64 Hz. Each step applies horizontal acceleration
toward the input, a buffered and coyote-timed jump, a jump cut when the button is
released, and gravity with a terminal speed. `move_and_collide` then moves the box
one axis at a time against the level's solid rectangles, clamping at the first
contact. Movement constants live in the `PlayerTuning` resource.

Rendering never moves the player. [visuals.rs](../../../src/visuals.rs) interpolates
the `Transform` between `PreviousPosition` and `Position` with
`Time<Fixed>::overstep_fraction`, so motion is smooth at any display rate.

## Consequences

- Tests in [tests.rs](../../../src/tests.rs) run the real movement headless with
  `MinimalPlugins` and `TimeUpdateStrategy::ManualDuration`, one fixed step per
  update, and assert exact landings and jump heights.
- No physics dependency; build times and Bevy upgrades stay simpler.
- Collision is limited to axis-aligned rectangles. Each step must move less than a
  tile to avoid tunnelling; the default tuning moves at most about 14 units per step
  against 32-unit tiles. Slopes, rotation, and moving platforms need new code or an
  engine.
- `PlayerTuning` values are play-test values; tests guard ranges such as jump
  height and gap clearance, not feel.

## The tell

- `cargo test --locked` passes the movement tests in `src/tests.rs`. A failure after
  a tuning change means the level may no longer be completable as designed.
- `rg -n "Transform" src/player.rs src/run.rs src/simulation.rs` finds nothing: the
  simulation uses `Position`, and only presentation code writes transforms.
- `rg -n "avian|rapier" Cargo.toml` finds nothing while this record is accepted.

These checks cannot judge feel; play the level after changing tuning.
