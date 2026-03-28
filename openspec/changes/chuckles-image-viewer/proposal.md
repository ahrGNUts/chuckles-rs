## Why

Linux lacks a lightweight image viewer that correctly handles file-manager integration. Eye of GNOME (eog) opens the wrong image when double-clicked from Nemo (defaulting to the first alphabetical file), navigates in alphabetical order regardless of user preference, and skips unsupported formats during navigation. Loupe (GNOME's replacement) improves on format support but still lacks power-user features like configurable sort order, metadata panels, and basic editing. This project fills that gap: a fast, correct image viewer built in Rust with GTK4.

## What Changes

- New standalone application: `chuckles` -- a lightweight image viewer for Linux
- Correct file opening: always displays the exact image passed by the file manager
- Directory navigation with natural filename sort (default) and configurable alternatives
- Two viewing modes: windowed (GTK4 with header bar, sidebar, thumbnails) and full-screen with fly-out edge panels
- Comprehensive metadata display: file info, EXIF summary, and raw tag dump
- Geometric editing: lossless JPEG rotation/flip, crop with aspect ratio presets, resize with Lanczos3 resampling
- Safe save behavior: Ctrl+Shift+S triggers Save-As when edits are pending, never silently overwrites originals
- Broad format support via pure Rust libraries: JPEG, PNG, GIF, BMP, WebP, TIFF, JPEG XL, SVG
- XDG-compliant configuration via TOML
- Desktop entry for Nemo/freedesktop integration

## Capabilities

### New Capabilities
- `image-viewing`: Core image display, zoom, pan, fit-to-window, and actual-size modes
- `directory-navigation`: Scan parent directory, natural sort, configurable sort modes, prev/next/first/last traversal
- `file-opening`: Desktop entry integration, correct single-file opening from file managers, async directory scanning
- `windowed-mode`: GTK4 windowed UI with header bar, image canvas, metadata sidebar, thumbnail strip
- `fullscreen-mode`: Zero-chrome full-screen view with fly-out edge panels (thumbnails, metadata, tools, navigation)
- `metadata-display`: File info panel, EXIF/camera data summary, scrollable raw metadata tag dump
- `geometric-editing`: Lossless JPEG rotate/flip, interactive crop with aspect ratio presets, resize/resample dialog
- `format-support`: Image decoding pipeline supporting JPEG, PNG, GIF, BMP, WebP, TIFF, JXL, SVG via pluggable decoders
- `configuration`: XDG-compliant TOML config for sort preferences, UI state, mouse behavior, zoom defaults
- `keyboard-navigation`: Keyboard shortcuts for all core operations (navigation, zoom, mode toggle, editing, panels)

### Modified Capabilities

(none -- greenfield project)

## Impact

- **New binary**: `chuckles` Rust application
- **Dependencies**: `gtk4-rs`, `image` crate, `jxl-oxide`, `resvg`, `kamadak-exif` (or similar EXIF library), `alphanumeric-sort` (natural sort), `imagesize` (fast header-only dimension reading), `toml`/`serde` for config
- **System integration**: `.desktop` file registering for image MIME types, installable via standard Linux packaging
- **Build requirements**: Rust toolchain, GTK4 development libraries (`libgtk-4-dev` or equivalent)
