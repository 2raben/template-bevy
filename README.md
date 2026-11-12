# Bevy template

A starting point for 2D games built with [Bevy](https://bevy.org) 0.20. It ships a
small, tested platformer, a documentation structure for shaping and recording work,
agent instructions with a library of Bevy skills, and [mise](https://mise.jdx.dev)
tasks to build, test, and run the game.

## What's inside

| Path | Purpose |
| --- | --- |
| `src/` | The starter platformer: headless simulation plus presentation plugins |
| `docs/idea.md` | Game direction and longer-term possibilities |
| `docs/pitches/` | Shaped feature proposals (Problem, Solution, Security, Rabbit Holes, No-Gos) |
| `docs/architecture/` | Current structure and conventions, and [decision records](docs/architecture/decisions/README.md) |
| `.agents/skills/` | Bevy 0.20 skills for coding agents; versions in `skills-lock.json` |
| `shell.nix` | Native build and runtime libraries for NixOS (`nix-shell`) |
| `.mise/` | Pinned Rust toolchain and `build`, `test`, `run`, `check`, `fmt`, `agents:init-claude` tasks |
| `AGENTS.md`, `CONTRIBUTING.md` | Working brief for agents; workflow and validation for everyone |

## Starting a new game from this template

1. Rename the package in `Cargo.toml` and the window title in `src/main.rs`.
2. Replace [docs/idea.md](docs/idea.md) with your game's direction.
3. Shape the first feature as a [pitch](docs/pitches/README.md). Keep the starter
   pitch as an example or delete it with the code it describes.
4. Review the [ADRs](docs/architecture/decisions/README.md): keep those that still
   hold, and supersede those you change.
5. Rewrite the project description in this README, `AGENTS.md`, and the manual
   checks in `CONTRIBUTING.md` as the game takes shape.

## How to play the starter

Run right, collect the coins, and touch the flag. Falling out of the level returns
you to the start; Restart begins a fresh run with all coins back and the clock reset.

| Action | Keyboard | Controller |
| --- | --- | --- |
| Run left / right | A / D or Left / Right | Left stick or D-pad |
| Jump (hold for higher) | Space, W, or Up | South (A / Cross) |
| Restart | R or Backspace | Select |
| Quit | Escape | – |

Keys are physical positions, so WASD stays in place on any keyboard layout. A jump
pressed just before landing, or just after running off a ledge, still counts.

## Requirements

- [mise](https://mise.jdx.dev), recommended: `mise install` provides Rust 1.97.1
  with rustfmt and Clippy, as pinned in [.mise/config.toml](.mise/config.toml).
  Without mise, install Rust 1.97.1 or newer (edition 2024) with rustup.
- Bevy 0.20.1, downloaded automatically by Cargo. `Cargo.lock` pins dependencies.
- Clang and LLD on x86_64 Linux, used by [.cargo/config.toml](.cargo/config.toml)
  for faster linking. Remove that file to use the default linker instead.
- A working Vulkan (Linux, Windows) or Metal (macOS) graphics driver.

### Linux native libraries

Bevy needs ALSA, udev, X11 and Wayland libraries, and pkg-config. On Arch Linux:

```sh
sudo pacman -S --needed base-devel pkgconf alsa-lib systemd \
  libx11 libxcursor libxrandr libxi libxkbcommon wayland clang lld \
  vulkan-icd-loader
```

On Debian or Ubuntu:

```sh
sudo apt install build-essential pkg-config libasound2-dev libudev-dev \
  libx11-dev libxcursor-dev libxrandr-dev libxi-dev libxkbcommon-dev \
  libwayland-dev clang lld
```

On NixOS, adding these packages to your profile usually does not make them
visible to builds or the dynamic loader. If your system configuration does not
expose them to Cargo builds, use the project's [shell.nix](shell.nix); it provides
pkg-config, Clang, LLD, and the libraries, and puts the runtime libraries on
`LD_LIBRARY_PATH`. The Vulkan driver comes from the system
(`hardware.graphics.enable = true`):

```sh
nix-shell          # then: mise run run
nix-shell --run 'mise run check'
```

`nix-shell` is not entered automatically when you open the folder. To load it on
`cd`, install [direnv](https://direnv.net) with
[nix-direnv](https://github.com/nix-community/nix-direnv), add a `.envrc` containing
`use nix`, and run `direnv allow` once.

Install a Vulkan driver for your GPU on every distribution. If audio is routed through PipeWire, keep its ALSA plugin
(`pipewire-alsa`) installed.

See Bevy's [Linux dependency instructions](https://github.com/bevyengine/bevy/blob/latest/docs/linux_dependencies.md)
for other distributions and its [setup guide](https://bevy.org/learn/quick-start/getting-started/setup/).

## Development

Read [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution workflow and validation,
the [architecture documentation](docs/architecture/README.md) for current structure
and conventions, and the [ADRs](docs/architecture/decisions/README.md) for recorded
architectural choices. AI coding agents should start with [AGENTS.md](AGENTS.md).

```sh
mise install     # once: install the pinned toolchain
mise run run     # build and start the game
mise run test    # headless tests
mise run build   # debug build; `mise run build --release` for release
mise run check   # format, Clippy, build, and tests before a PR
mise tasks       # list every task
```

Each task passes extra arguments to Cargo, for example `mise run test jump`. The
tasks are plain scripts in [.mise/tasks/](.mise/tasks/), so the equivalent Cargo
commands also work:

```sh
cargo run --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked
cargo test --locked
```

The first build downloads and compiles Bevy and can take several minutes.
Dependencies are optimized in development builds for usable runtime performance.

## Agent skills

`.agents/skills/` holds Bevy 0.20 skills from
[chrisgliddon/bevy-skills](https://github.com/chrisgliddon/bevy-skills): ECS, input,
cameras, UI, assets, audio, animation, physics, rendering, testing, WebAssembly,
and more, plus `similarity-rs` for finding duplicated Rust code. Start with the
`bevy` router skill. [skills-lock.json](skills-lock.json) records each skill's
source and content hash; update them together.

Codex and other agents that follow the `AGENTS.md` convention read these files
directly. Claude Code reads `CLAUDE.md` and `.claude/skills/` instead; link them to
the shared files once per checkout:

```sh
mise run agents:init-claude
```

This creates `CLAUDE.md -> AGENTS.md` and `.claude/skills -> ../.agents/skills`.
Rerunning it is safe, and it never replaces an existing file or a link that points
elsewhere.
