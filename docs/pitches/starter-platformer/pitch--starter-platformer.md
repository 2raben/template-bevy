# Problem

A new Bevy project begins with an empty window and many open questions: where
gameplay code lives, how input reaches the simulation, how to test without a
window, and how decisions get recorded. Answering them while also designing a game
slows the first playable experiment, and an empty template answers none of them.

The template needs a game small enough to read in one sitting and to delete
without regret, yet complete enough to show each convention in use.

# Solution

One short side-scrolling level with a single player character.

## Run and jump

The player is a coloured box with an eye that shows which way it faces. Left and
right run with quick acceleration; jump launches from the ground. Holding jump
gives a full jump of about three and a half tiles; tapping it gives a short hop.
Two forgiving timers keep jumps from feeling dropped: a jump pressed just before
landing still fires, and a jump pressed just after walking off a ledge still
counts.

Keyboard: A/D or arrow keys to run, Space/W/Up to jump, R or Backspace to restart,
Escape to quit. A controller's left stick or D-pad runs, South jumps, and Select
restarts.

## One level

The level is a text map in the source, one character per tile: ground, the start,
the goal, and coins. It introduces a small gap, a stair, a floating platform with
coins above it, a wall to jump onto, and a wider four-tile gap before the flag. The
camera follows the player and stays inside the level.

## A run

Coins disappear when touched and are counted. Touching the flag finishes the run:
the clock stops, movement stops, and a centred message shows the result and how to
play again. Falling below the level returns the player to the start without
resetting coins or time; Restart resets everything.

## Built to be tested

The level, movement, and run rules run headless on Bevy's fixed timestep. Tests
drive them through the same `Controls` resource that the keyboard and controller
write, so they check the real behaviour: landing, running, jump heights, the
buffered jump, walls, coins, the goal, falling out, and restart.

# Security

No network access, file storage, or external content. The level is compiled into
the binary.

# Rabbit Holes

- **Physics engines.** Rigid-body physics is unnecessary for axis-aligned boxes and
  would hide the movement code a template should show. Use a small hand-written
  move-and-collide on tile rectangles (see
  [ADR 0001](../../architecture/decisions/0001-move-the-player-with-hand-written-kinematics.md)).
- **Tunnelling and slopes.** Keep speeds small enough per fixed step that a box
  never skips a tile, and keep all geometry axis-aligned.
- **Level formats.** A text map is enough; asset loading, editors, and
  serialization are left for real projects.
- **Art.** Coloured sprites only; no textures, animation, or fonts to manage.

# No-Gos

No enemies, hazards, moving platforms, sound, menus, settings, saved progress,
multiple levels, or level loading from files.
