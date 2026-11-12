# Game idea — Starter Platformer

This template ships a deliberately small 2D platformer so a new project starts from
something that builds, runs, and has tests. Replace this document with your own
game's direction when you start a project from the template.

## What it is

Run and jump through one short side-scrolling level, pick up coins on the way, and
touch the flag at the end. Falling out of the level puts you back at the start;
Restart begins a fresh run. A small panel shows the collected coins and the time.

## Why this starter

- **One of everything.** The game has a body that moves, a level, something to
  collect, a goal, a camera, and a HUD: enough to show where each kind of code goes
  without committing a new project to any of them.
- **Responsive feel without a physics engine.** Coyote time, jump buffering, and a
  variable jump height make the controls forgiving in a few dozen lines.
- **Headless by default.** The rules run without a window, so the tests exercise
  the real systems.

## Directions a project might take

These are examples, not commitments:

- Levels loaded as assets instead of the built-in text map.
- Hazards, moving platforms, or enemies.
- Sprites, animation, sound, and particles.
- Menus, settings, and saved progress.

Shape each larger step as a [pitch](pitches/README.md) before building it.
