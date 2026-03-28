## 1. Project Scaffolding

- [ ] 1.1 Initialize Cargo project with `cargo init`, add dependencies to Cargo.toml (gtk4, image, jxl-oxide, resvg, kamadak-exif, natord, toml, serde, gio)
- [ ] 1.2 Set up basic GTK4 application entry point (Application, ApplicationWindow, activate signal)
- [ ] 1.3 Create module structure: main.rs, app.rs, viewer/, editing/, metadata/, config/, formats/

## 2. Image Decoding Pipeline

- [ ] 2.1 Implement format detection by magic bytes (content sniffing, not extension-only)
- [ ] 2.2 Implement core format decoder using the `image` crate (JPEG, PNG, GIF, BMP, WebP, TIFF)
- [ ] 2.3 Implement JPEG XL decoder integration via `jxl-oxide`
- [ ] 2.4 Implement SVG renderer integration via `resvg`
- [ ] 2.5 Implement decoder trait abstraction for pluggable format support
- [ ] 2.6 Implement graceful error handling for corrupt/unsupported files with user-visible error messages

## 3. Directory Scanning & Navigation

- [ ] 3.1 Implement directory scanner that filters to supported image format extensions
- [ ] 3.2 Implement natural filename sort using `alphanumeric-sort`
- [ ] 3.3 Implement additional sort modes (date modified, file size, file type, dimensions); use `imagesize` crate for fast header-only dimension reading, with fallback to format-specific decoders for JXL/SVG
- [ ] 3.4 Implement ascending/descending sort toggle
- [ ] 3.5 Implement async directory scanning via `gio::spawn_blocking` (non-blocking main thread)
- [ ] 3.6 Implement file list index management (current position, next, prev, first, last)
- [ ] 3.7 Implement adjacent image preloading in background (prev and next)

## 4. File Opening & CLI Integration

- [ ] 4.1 Implement command-line argument parsing (single file path via `%f`)
- [ ] 4.2 Implement immediate display of the argument file before directory scan completes
- [ ] 4.3 Implement file chooser dialog for bare launch (no arguments)
- [ ] 4.4 Create .desktop file with MIME type registrations for all supported formats

## 5. Image Canvas & Viewing

- [ ] 5.1 Implement image canvas widget with fit-to-window default zoom
- [ ] 5.2 Implement actual size (100%) zoom mode
- [ ] 5.3 Implement incremental zoom in/out with keyboard (+/-) and Ctrl+scroll
- [ ] 5.4 Implement click-and-drag panning for zoomed images
- [ ] 5.5 Implement configurable scroll wheel behavior (navigate vs zoom)

## 6. Windowed Mode UI

- [ ] 6.1 Build GTK4 HeaderBar with filename, image index/total, sort selector, zoom controls, edit toggle
- [ ] 6.2 Implement central image canvas layout that fills available space
- [ ] 6.3 Implement toggleable metadata sidebar (right side, `I` key)
- [ ] 6.4 Implement toggleable thumbnail strip (bottom, `T` key) with lazy thumbnail loading
- [ ] 6.5 Implement toggleable edit tools panel (rotate, flip, crop, resize buttons) via header bar "Edit" toggle
- [ ] 6.6 Implement navigation arrow overlays on canvas hover
- [ ] 6.7 Implement click-on-thumbnail navigation
- [ ] 6.8 Implement window state persistence (size, position, sidebar/thumbnail visibility)

## 7. Full-Screen Mode

- [ ] 7.1 Implement full-screen toggle via Enter/F11/double-click with Escape to exit
- [ ] 7.2 Implement edge detection zones for fly-out panel triggers
- [ ] 7.3 Implement fly-out thumbnail strip + navigation bar at bottom edge using GtkOverlay + GtkRevealer
- [ ] 7.4 Implement fly-out metadata panel at right edge
- [ ] 7.5 Implement fly-out edit tools panel at left edge
- [ ] 7.6 (merged into 7.3 -- navigation controls integrated in bottom thumbnail panel)
- [ ] 7.7 Tune panel animation timing and edge sensitivity

## 8. Metadata Display

- [ ] 8.1 Implement file info section (filename, path, size, dimensions, color depth, format)
- [ ] 8.2 Implement EXIF parsing via `kamadak-exif` for camera summary fields
- [ ] 8.3 Implement scrollable raw metadata tag dump (all EXIF/XMP/IPTC tags)
- [ ] 8.4 Implement metadata panel update on image navigation
- [ ] 8.5 Handle images with no metadata gracefully (hide sections, show "No EXIF data")

## 9. Geometric Editing

- [ ] 9.1 Implement rotation (90 CW/CCW) and flip (H/V) via decode-transform-reencode at maximum quality for all formats
- [ ] 9.2 (reserved for future lossless JPEG rotation/flip via DCT coefficient manipulation)
- [ ] 9.3 Implement interactive crop mode with draggable selection rectangle and aspect ratio presets
- [ ] 9.4 (reserved for future lossless JPEG crop indicator)
- [ ] 9.5 Implement resize dialog with dimension/percentage input and locked aspect ratio
- [ ] 9.6 Implement Lanczos3 resampling for resize operations
- [ ] 9.7 Implement Save-As dialog (Ctrl+Shift+S opens file chooser, never overwrites silently)
- [ ] 9.8 Implement unsaved changes prompt when navigating away from edited image

## 10. Keyboard & Input Handling

- [ ] 10.1 Implement navigation shortcuts (Left/Right arrows, Home/End)
- [ ] 10.2 Implement zoom shortcuts (+, -, 1, F)
- [ ] 10.3 Implement mode toggle shortcuts (Enter, F11, Escape)
- [ ] 10.4 Implement editing shortcuts (L, R, H, V, X)
- [ ] 10.5 Implement panel toggle shortcuts (I, T)
- [ ] 10.6 Implement Ctrl+Q quit and Ctrl+Shift+S Save-As
- [ ] 10.7 Implement mouse: double-click full-screen toggle, Ctrl+scroll zoom, click-drag pan

## 11. Configuration

- [ ] 11.1 Implement XDG-compliant config file path resolution
- [ ] 11.2 Define config struct with serde and TOML parsing with defaults for all fields
- [ ] 11.3 Implement config loading at startup (graceful handling of missing/malformed file)
- [ ] 11.4 Implement window state save on application close (only write to config file if it already exists; MUST NOT auto-create config file)
- [ ] 11.5 Wire config values to sort mode, scroll wheel behavior, zoom default, and background color

## 12. Integration & Polish

- [ ] 12.1 End-to-end test: open image from file manager, verify correct image displayed
- [ ] 12.2 Test navigation through directory with mixed supported/unsupported files
- [ ] 12.3 Test all keyboard shortcuts in both windowed and full-screen modes
- [ ] 12.4 Test edit workflow: rotate, crop, resize, Save-As
- [ ] 12.5 Test with directories of 500+ images for reasonable performance
- [ ] 12.6 Verify .desktop file MIME type registration works with Nemo
