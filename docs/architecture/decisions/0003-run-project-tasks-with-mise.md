---
tags: [adr]
status: accepted
date: 2026-11-12
sources:
  - .mise/config.toml
  - .mise/tasks/build
  - .mise/tasks/test
  - .mise/tasks/run
  - .mise/tasks/check
  - .mise/tasks/fmt
  - .mise/tasks/agents/init-claude
  - Cargo.toml
---

# 0003 — Pin the toolchain and run project tasks with mise

## Context

Contributors and agents need the same Rust version as `rust-version` in
`Cargo.toml` and the same validation commands. Remembering `--locked`, Clippy
flags, and the order of checks is error-prone, and machines differ in how Rust is
installed.

Alternatives considered:

- **Plain Cargo commands in the docs.** No extra tool, but every reader repeats the
  flags and the toolchain is not pinned.
- **`rust-toolchain.toml` plus a `justfile` or `Makefile`.** Pins Rust and names the
  commands, but needs two mechanisms and another tool for tasks.
- **mise** with a project config and file tasks: one tool pins Rust (through rustup)
  and runs named tasks written as ordinary shell scripts.

## Decision

[.mise/config.toml](../../../.mise/config.toml) pins Rust 1.97.1 with rustfmt and
Clippy. Executable scripts in `.mise/tasks/` define the tasks: `build`, `test`,
`run`, `check` (format, Clippy, build, tests), and `fmt`. Each passes extra
arguments to Cargo, for example `mise run build --release`. Tasks call the same
Cargo commands documented in [CONTRIBUTING.md](../../../CONTRIBUTING.md), so mise
stays optional.

Tasks in a subdirectory are namespaced: `agents/init-claude` runs as
`mise run agents:init-claude` and links Claude Code's `CLAUDE.md` and
`.claude/skills` to the shared `AGENTS.md` and `.agents/skills`.

## Consequences

- `mise install` provides the right toolchain; `mise run check` runs every
  contribution check.
- Task scripts are plain Bash and readable without mise.
- The Rust version is recorded in both `Cargo.toml` and `.mise/config.toml`; bump
  them together.
- Native libraries (audio, input, windowing, Vulkan) are not managed by mise; the
  README lists them per platform, and [shell.nix](../../../shell.nix) provides them
  on NixOS. mise supplies Rust inside that shell as well.

## The tell

- `rg -n "rust-version|rust = " Cargo.toml .mise/config.toml` shows the same version.
- `mise tasks` lists `agents:init-claude`, `build`, `check`, `fmt`, `run`, and `test`.
- `rg -n "cargo " .mise/tasks` matches the commands in `CONTRIBUTING.md`; review
  both when either changes.
