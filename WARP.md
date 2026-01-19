# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WezTerm is a GPU-accelerated cross-platform terminal emulator and multiplexer written in Rust. It supports macOS, Linux (X11/Wayland), and Windows.

## Build Commands

```bash
# Install system dependencies (Linux only)
./get-deps

# Build all main binaries
cargo build -p wezterm -p wezterm-gui -p wezterm-mux-server

# Quick type checking during development
cargo check

# Run tests
cargo nextest run                           # All tests
cargo nextest run -p wezterm-escape-parser  # no_std crate tests
cargo test --all                            # Alternative: standard cargo test

# Format code (requires nightly toolchain)
cargo +nightly fmt

# Build and serve documentation
ci/build-docs.sh serve
```

## Code Architecture

### Core Crates

- **`term/`** - Core terminal model, agnostic of windowing. Handles escape sequences, screen buffer, and terminal state. Aims for xterm compatibility (see [xterm ctlseqs](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html)).

- **`termwiz/`** - Terminal wizard library providing:
  - Terminal input parsing and keyboard handling
  - Line editing and widgets
  - Terminal rendering abstractions

- **`mux/`** - Terminal multiplexer layer managing:
  - Panes (`pane.rs`, `localpane.rs`) - Individual terminal instances
  - Tabs (`tab.rs`) - Container for split panes
  - Windows (`window.rs`) - Container for tabs
  - Domains (`domain.rs`) - Local, SSH, tmux connections
  - SSH integration (`ssh.rs`) and tmux control mode (`tmux*.rs`)

- **`window/`** - Platform-specific window abstraction:
  - `os/macos/` - macOS Cocoa implementation
  - `os/x11/` - X11 implementation
  - `os/wayland/` - Wayland implementation
  - `os/windows/` - Windows implementation
  - `egl.rs` - OpenGL/EGL context management

- **`config/`** - Configuration system:
  - `config.rs` - Main configuration struct
  - `lua.rs` - Lua configuration loading
  - `keyassignment.rs` - Key binding actions

### Binary Crates

- **`wezterm/`** - CLI tool for interacting with wezterm instances
- **`wezterm-gui/`** - Main GUI application
  - `termwindow/` - Main terminal window implementation
  - `commands.rs` - Command palette and key binding commands
  - `glyphcache.rs`, `shapecache.rs` - Font glyph caching
  - GPU rendering via wgpu (`shader.wgsl`)
- **`wezterm-mux-server/`** - Headless multiplexer server

### Supporting Crates

- **`wezterm-font/`** - Font discovery, loading, and shaping (harfbuzz, freetype)
- **`codec/`** - Serialization for mux client/server protocol
- **`lua-api-crates/`** - Lua API bindings organized by feature area
- **`deps/`** - Vendored dependencies (cairo, freetype, harfbuzz, fontconfig)

## Testing

```bash
# Run specific package tests
cargo test -p wezterm-term
cargo test -p termwiz

# Run single test by name
cargo test -p wezterm-term test_name
```

Terminal behavior tests use helper classes in `term/src/test/`.

## Key Data Flow

1. **Input**: `window/` captures keyboard/mouse → `wezterm-gui/src/termwindow/keyevent.rs` processes → `config/keyassignment.rs` maps to actions
2. **Terminal**: PTY output → `term/` parses escape sequences → updates screen buffer
3. **Rendering**: `mux/` provides pane content → `wezterm-gui/termwindow/render/` draws via WebGPU/OpenGL

## Code Style

- Uses Rust 2018 edition
- 4-space indentation
- Module-level import granularity
- Run `cargo +nightly fmt` before commits

## Platform-Specific Notes

- **macOS**: Metal rendering, Cocoa windowing
- **Linux**: X11 or Wayland, fontconfig for fonts
- **Windows**: DirectWrite fonts, ConPTY
