# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

chuckles-rs is a lightweight Linux image viewer written in Rust with a GTK4 UI. It targets Nemo (Cinnamon file manager) as the primary integration point, solving eog's broken file-opening behavior (wrong image on double-click, alphabetical-only navigation, format gaps).

**License:** GPL-3.0

## Build & Development

This is a Rust/Cargo project using GTK4. Build requirements:
- Rust toolchain (stable)
- GTK4 development libraries (`libgtk-4-dev` on Debian/Ubuntu, `gtk4-devel` on Fedora)

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo run                # Run (no args = file chooser dialog)
cargo run -- /path/to/image.jpg  # Open specific image
cargo test               # Run tests
cargo clippy             # Lint
cargo fmt --check        # Check formatting
```

## Architecture

Key design decisions (see `openspec/changes/chuckles-image-viewer/design.md` for full rationale):

- **UI:** GTK4 via `gtk4-rs`. Two modes: windowed (header bar, sidebar, thumbnails) and full-screen (fly-out edge panels via `gtk4::Overlay` + `gtk4::Revealer`)
- **Async:** Image loading and directory scanning use `glib::MainContext` / `gio::spawn_blocking` — no tokio. Main thread never blocks on I/O.
- **Formats:** `image` crate for JPEG/PNG/GIF/BMP/WebP/TIFF, `jxl-oxide` for JXL, `resvg` for SVG. All pure Rust — no C dependencies for core formats. AVIF/HEIC/RAW deferred behind future feature flags.
- **Sorting:** `alphanumeric-sort` crate for natural filename sort (default). `imagesize` crate for fast header-only dimension reading when sorting by dimensions.
- **Editing:** Geometric only (rotate, flip, crop, resize). Decode-transform-reencode at max quality. Always Save-As via Ctrl+Shift+S — never overwrites originals.
- **Config:** TOML at `$XDG_CONFIG_HOME/chuckles/config.toml`. App works with no config file; MUST NOT auto-create one.
- **Instances:** Each invocation is independent — no D-Bus single-instance logic.

## Specs & Requirements

This project uses [OpenSpec](https://github.com/Fission-AI/OpenSpec) for spec-driven development. All specifications live in `openspec/changes/chuckles-image-viewer/`:

- `proposal.md` — what and why
- `design.md` — architecture decisions and trade-offs
- `tasks.md` — implementation checklist
- `specs/` — 10 capability specs with WHEN/THEN scenarios (image-viewing, directory-navigation, file-opening, windowed-mode, fullscreen-mode, metadata-display, geometric-editing, format-support, configuration, keyboard-navigation)

Use `/opsx:apply` to implement tasks from the spec. Consult the relevant spec file before implementing any feature.

## Key Constraints

- File opened from Nemo MUST display immediately, before directory scanning completes
- Escape exits full-screen only — never closes the application. Ctrl+Q quits.
- Keyboard shortcuts work regardless of whether the edit tools panel is visible
- Directory scanning filters by file extension only; format detection by magic bytes applies only during decoding
- Ctrl+scroll always zooms, regardless of scroll wheel mode setting
