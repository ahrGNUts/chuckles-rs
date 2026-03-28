## Context

There is no existing codebase -- chuckles-rs is a greenfield Rust project. The target platform is Linux desktops using GTK4, with Nemo (Cinnamon's file manager) as the primary integration target. The freedesktop Desktop Entry specification defines how file managers launch image viewers: a single file path is passed via `%f`, with no sort-order metadata. This architectural constraint means the viewer must independently scan and sort directory contents.

The Rust ecosystem provides mature libraries for image decoding (`image` crate), EXIF parsing, SVG rendering (`resvg`), and GTK4 bindings (`gtk4-rs`).

## Goals / Non-Goals

**Goals:**
- Always display the exact image the user clicked in the file manager
- Navigate directory contents in natural filename sort order by default, with configurable alternatives
- Provide two viewing modes: windowed (standard GTK4 app) and full-screen with fly-out edge panels
- Display comprehensive image metadata (file info, EXIF summary, raw tags)
- Support geometric editing operations: lossless JPEG rotation/flip, crop, resize
- Support JPEG, PNG, GIF, BMP, WebP, TIFF, JPEG XL, and SVG out of the box
- Never silently overwrite original files (Save-As by default)
- Maintain reasonable performance for directories with hundreds of images

**Non-Goals:**
- Pixel-level editing (brightness, contrast, color correction, clone stamp, red-eye removal)
- Photo management/cataloging (import, albums, tags, ratings)
- Batch operations (batch rename, batch convert)
- AVIF, HEIC/HEIF, or RAW camera format support in v1 (require C dependencies)
- Thumbnail caching to disk
- Per-directory sort preference persistence
- Slideshow mode
- Image comparison (side-by-side)
- Plugin/extension system

## Decisions

### D1: GTK4 via gtk4-rs for the UI toolkit

GTK4 is the native toolkit for Linux desktop applications and integrates naturally with Nemo and GNOME/Cinnamon environments. Alternatives considered:
- **Iced / egui (pure Rust)**: More Rust-idiomatic but lack mature desktop integration (file dialogs, desktop entry, system theme). Would require significantly more custom work for basic desktop features.
- **Qt via cxx-qt**: Good toolkit but adds C++ build complexity and is less natural on Cinnamon/GNOME desktops.
- **Slint**: Promising but still maturing for full desktop applications.

GTK4 provides the header bar, file chooser, keyboard/mouse event handling, and CSS theming that the UI design requires.

### D2: Async image loading via glib async/spawn

Image decoding and directory scanning run on background threads using `glib::MainContext` futures and `gio::spawn_blocking`. This avoids a separate async runtime dependency (`tokio`) while integrating naturally with the GTK event loop. The main thread never blocks on I/O.

Alternatives considered:
- **tokio**: Powerful but overkill for this use case and adds friction with GTK's event loop.
- **Synchronous loading**: Simple but would freeze the UI on large images or slow storage.

### D3: image crate as the primary decoder with pluggable format extensions

The `image` crate provides a unified API for JPEG, PNG, GIF, BMP, WebP, and TIFF. JPEG XL is added via `jxl-oxide` (pure Rust) and SVG via `resvg` (pure Rust). This keeps the dependency tree C-free for core formats.

The decoder pipeline uses a trait-based design so AVIF (`libavif`), HEIC (`libheif-rs`), and RAW (`rawloader`) can be added later behind feature flags without modifying core code. AVIF is deferred because the `image` crate's AVIF support requires C dependencies (`dav1d`/`rav1e`), which conflicts with the goal of keeping core format dependencies C-free.

### D4: Natural sort as default directory ordering

Natural sort (img1, img2, ..., img10 instead of img1, img10, img2) matches user expectations and is what most file managers use when sorting by name. The `alphanumeric-sort` crate provides this (actively maintained, first-class `Path`/`OsStr` support, zero dependencies). Other sort modes (date modified, file size, type, dimensions) are available via a menu/hotkey.

### D5: Full-screen fly-out panels via GTK4 Overlay and Revealer

The full-screen mode uses `gtk4::Overlay` to layer four `gtk4::Revealer` panels over the image canvas. Each panel animates in when the mouse enters an edge detection zone and retracts when it leaves. This is implemented purely in GTK4 without custom rendering.

### D6: Geometric operations via decode-transform-reencode

All geometric operations (rotate, flip, crop, resize) decode the image, apply the transformation in memory, and re-encode at maximum quality on save. This approach is simple and works uniformly across all formats. A future enhancement may add lossless JPEG operations via DCT coefficient manipulation (wrapping libjpeg-turbo or adopting a pure-Rust library when available), but no mature pure-Rust solution currently exists for this.

### D7: XDG Base Directory compliance for configuration

Config lives at `$XDG_CONFIG_HOME/chuckles/config.toml` (defaulting to `~/.config/chuckles/config.toml`). All settings have sensible defaults; the app works with no config file. Parsed via `toml` + `serde`.

### D8: Multiple instances (no single-instance enforcement)

Each file-manager invocation spawns a new process/window. No D-Bus single-instance logic. This is simpler to implement, avoids IPC complexity, and matches user expectations when opening multiple images from a file manager.

## Risks / Trade-offs

- **GTK4 fly-out panel complexity** → The Overlay+Revealer approach may require careful tuning of hover detection zones to feel responsive without accidental triggers. Mitigation: configurable edge sensitivity, test with real usage patterns early.
- **Format support gaps in pure Rust decoders** → The `image` crate, `resvg`, and `jxl-oxide` cover common cases well but may fail on exotic variants (unusual TIFF compression, complex SVGs, advanced JXL features like HDR tone mapping or some progressive JPEG reconstruction paths). Mitigation: graceful error display with format/error details rather than crashes. Log unsupported variants for future improvement.
- **EXIF library maturity** → Rust EXIF libraries (`kamadak-exif`, `rexif`) are functional but less battle-tested than C equivalents. Some camera-specific tags may be missing or misinterpreted. Mitigation: display raw tag values alongside interpreted values so users can still access data.
- **JPEG re-encoding quality loss** → Geometric operations on JPEGs use decode-transform-reencode at maximum quality, which introduces minimal but nonzero quality loss. Mitigation: save always uses maximum quality; lossless JPEG operations via DCT coefficient manipulation are a planned future enhancement pending a mature pure-Rust library or acceptable C binding.
- **Performance with large directories** → v1 uses in-memory thumbnail generation without disk caching. Directories with thousands of images will have a cold-start delay. Mitigation: lazy thumbnail loading (only generate visible thumbnails), preload adjacent images for navigation. Disk caching is a planned future optimization.
