## ADDED Requirements

### Requirement: Core format support via image crate
The application SHALL decode and display the following formats natively via the Rust `image` crate: JPEG, PNG, GIF (first frame), BMP, WebP, and TIFF.

#### Scenario: Open each core format
- **WHEN** the user opens a valid JPEG, PNG, GIF, BMP, WebP, or TIFF file
- **THEN** the image is decoded and displayed correctly

#### Scenario: Animated GIF displays first frame
- **WHEN** the user opens an animated GIF
- **THEN** the first frame is displayed as a static image

### Requirement: JPEG XL support
The application SHALL decode and display JPEG XL (.jxl) files using the `jxl-oxide` pure Rust decoder.

#### Scenario: Open JPEG XL file
- **WHEN** the user opens a .jxl file
- **THEN** the image is decoded and displayed correctly

### Requirement: SVG support
The application SHALL render and display SVG files using `resvg`. SVGs are rasterized at a resolution appropriate for the display/zoom level.

#### Scenario: Open SVG file
- **WHEN** the user opens an .svg file
- **THEN** the SVG is rendered and displayed at appropriate resolution

### Requirement: Graceful handling of unsupported or corrupt files
The application SHALL display a clear error message when a file cannot be decoded, including the format name and error details. The application MUST NOT crash on corrupt or unsupported files.

#### Scenario: Corrupt image file
- **WHEN** the user opens a corrupt JPEG file that fails to decode
- **THEN** an error message is displayed with the filename and error description

#### Scenario: Unsupported format in directory
- **WHEN** navigating through a directory that contains an unsupported file that was incorrectly included
- **THEN** the application displays an error for that file and allows navigation to continue

### Requirement: Format detection by content
The application SHALL detect image format by file content (magic bytes), not solely by file extension. A .jpg file containing PNG data SHALL be decoded as PNG.

#### Scenario: Mismatched extension
- **WHEN** the user opens a file named photo.jpg that actually contains PNG data
- **THEN** the file is decoded as PNG and displayed correctly
