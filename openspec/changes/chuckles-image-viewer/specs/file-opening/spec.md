## ADDED Requirements

### Requirement: Open the exact file passed as argument
The application SHALL immediately display the image file specified as a command-line argument. The requested image MUST be the first image shown, regardless of its position in any sort order.

#### Scenario: Open image from file manager
- **WHEN** the user double-clicks photo_42.jpg in Nemo and chuckles is the default viewer
- **THEN** photo_42.jpg is immediately displayed, not the first alphabetical image

#### Scenario: File argument is the only required argument
- **WHEN** chuckles is launched with a single file path argument
- **THEN** that file is decoded and displayed before any directory scanning begins

### Requirement: Async directory scanning on open
The application SHALL scan the parent directory of the opened file in the background after displaying the requested image. Directory scanning MUST NOT block or delay the initial image display.

#### Scenario: Large directory
- **WHEN** the user opens an image in a directory with 2000 files
- **THEN** the opened image is displayed immediately, and the directory file list populates in the background

#### Scenario: Directory scan completes
- **WHEN** the background directory scan finishes
- **THEN** the navigation index is positioned at the opened file and prev/next navigation becomes available

### Requirement: Launch without arguments opens file chooser
The application SHALL present a file chooser dialog when launched without any file path argument.

#### Scenario: Bare launch
- **WHEN** chuckles is launched with no arguments
- **THEN** a GTK file chooser dialog is displayed, filtered to supported image formats

### Requirement: Multiple instances supported
Each invocation of chuckles SHALL open a new independent window and process. There is no single-instance enforcement.

#### Scenario: Open two images from file manager
- **WHEN** the user double-clicks image_a.jpg and then double-clicks image_b.jpg in Nemo
- **THEN** two separate chuckles windows are open, each showing their respective image

### Requirement: Desktop entry for file manager integration
The application SHALL provide a .desktop file that registers chuckles as a handler for supported image MIME types (image/jpeg, image/png, image/gif, image/bmp, image/webp, image/tiff, image/jxl, image/svg+xml), using `Exec=chuckles %f`.

#### Scenario: Set as default viewer
- **WHEN** the user sets chuckles as the default image viewer via xdg-mime or Nemo preferences
- **THEN** double-clicking any supported image format in Nemo launches chuckles with that file
