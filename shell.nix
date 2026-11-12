# Native build and runtime dependencies for Bevy on NixOS.
#
# Enter with `nix-shell`, then use the mise tasks as usual (`mise run run`).
# Rust itself comes from mise (.mise/config.toml), not from this shell.
# The Vulkan driver comes from the system (`hardware.graphics.enable = true`).
{
  pkgs ? import <nixpkgs> { },
}:
let
  # Newer nixpkgs moved the X11 libraries out of the deprecated `xorg` set.
  x11 = name: oldName: pkgs.${name} or pkgs.xorg.${oldName};

  # Libraries Bevy loads at runtime (windowing, input, audio, graphics).
  runtimeLibs = with pkgs; [
    alsa-lib
    udev
    libxkbcommon
    wayland
    vulkan-loader
    (x11 "libx11" "libX11")
    (x11 "libxcursor" "libXcursor")
    (x11 "libxi" "libXi")
    (x11 "libxrandr" "libXrandr")
  ];
in
pkgs.mkShell {
  # pkg-config finds ALSA and udev at build time; clang and lld are the linker
  # configured in .cargo/config.toml.
  nativeBuildInputs = with pkgs; [
    pkg-config
    clang
    lld
  ];
  buildInputs = runtimeLibs;
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;
}
