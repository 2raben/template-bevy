# Working in this Bevy template

A Rust 2024 game built with Bevy 0.20. The current implementation is the starter
platformer from the [starter platformer pitch](docs/pitches/starter-platformer/pitch--starter-platformer.md):
one level from a text map, a player who runs and jumps with hand-written kinematics,
coins, a goal flag, a following camera, and a HUD. Use the versions in `Cargo.toml`
and the dependencies pinned by `Cargo.lock`.

## Start here

Read [CONTRIBUTING.md](CONTRIBUTING.md) and the
[architecture documentation](docs/architecture/README.md) before changing code.
Consult the [architectural decision records (ADRs)](docs/architecture/decisions/README.md)
for significant choices and their rationale.

- `src/main.rs`: application composition and window configuration.
- `src/simulation.rs`, `level.rs`, `player.rs`, `run.rs`: the headless simulation
  (plugin and fixed-step ordering, the tile map, movement and collision, and the
  run rules for coins, goal, falling out, and restart).
- `src/input.rs`, `visuals.rs`, `camera.rs`, `hud.rs`: device input, drawing and
  render interpolation, the camera, and on-screen text.
- `src/tests.rs`: headless simulation tests of movement and the run rules.
- `Cargo.toml` and `.cargo/config.toml`: dependencies, build profiles, and Linux
  linker settings.
- `shell.nix`: native libraries for NixOS; run Cargo and mise tasks inside `nix-shell` there.
- `.mise/config.toml` and `.mise/tasks/`: the pinned toolchain and the `build`,
  `test`, `run`, `check`, and `fmt` tasks.
- [docs/idea.md](docs/idea.md): game direction and longer-term possibilities.
- [docs/pitches/](docs/pitches/README.md): bounded feature proposals and companion sketches.

## Skills

`.agents/skills/` holds Bevy 0.20 skills. Load the `bevy` router skill first for any
Bevy task; it points to the focused skill (ECS, input, cameras, UI, assets, physics,
testing, and so on). Prefer patterns from these skills and from nearby code over
memory of older Bevy versions, whose APIs differ. Use `similarity-rs` before adding
helpers that may already exist. Do not edit installed skills in place; their hashes
are recorded in `skills-lock.json`.

`CLAUDE.md` and `.claude/skills` are symlinks to `AGENTS.md` and `.agents/skills`,
created by `mise run agents:init-claude`. Edit the shared originals, not the links.

## Architecture and scope

Keep game state and behavior within Bevy's entity/component/system model. Keep
systems focused and introduce modules or plugins as implemented features need them.
Do not add a parallel game loop or speculative framework.

Distinguish implemented behavior from ideas and pitches. Keep simulation code free
of rendering and device input so it stays testable headless, and pass input to
fixed-step systems only through the `Controls` resource. Simulation positions live
in `Position`; only presentation code writes `Transform`. Treat movement tuning as
play-test values and judge changes in the running game.

Update architecture documentation alongside changes to structure or behavior.
Record lasting architectural choices with the
[ADR template](docs/architecture/decisions/_adr-template.md) and update the decision
index. Use proposed status for unimplemented decisions; preserve the rationale of
accepted records when amending or superseding them.

## Editing and validation

Follow nearby Rust code and rustfmt. Keep changes focused, preserve existing user
changes, and avoid unrelated formatting or dependency upgrades. Do not edit
generated files under `target/`. Keep `Cargo.lock` committed and use `--locked`
unless intentionally updating dependencies.

Follow the [contribution checks](CONTRIBUTING.md#setup-and-validation). For code
changes, run `mise run check` (or the equivalent Cargo commands): formatting,
Clippy, build, and tests. Verify visual or input changes with `mise run run` when a
graphical environment is available. Add focused behavior or regression tests where
they provide useful coverage; do not claim a successful build proves gameplay works.

Documentation-only changes need source and relative-link checks plus
`git diff --check`, not a full build. Report actual checks, failures, and unavailable
coverage. Follow contribution rules for branches, Conventional Commits, and PRs.
