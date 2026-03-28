## ADDED Requirements

### Requirement: File info section
The metadata panel SHALL display basic file information: filename, file path, file size, image dimensions (width x height), color depth, and format name.

#### Scenario: Display file info for JPEG
- **WHEN** a JPEG image is loaded
- **THEN** the metadata panel shows filename, path, file size (e.g., "4.2 MB"), dimensions (e.g., "4032 x 3024"), color depth, and format ("JPEG")

### Requirement: EXIF camera data section
The metadata panel SHALL display a summary of common EXIF camera fields when present: camera make/model, lens, aperture, shutter speed, ISO, focal length, date taken, and GPS coordinates (if available).

#### Scenario: Image with EXIF data
- **WHEN** a JPEG image with EXIF metadata is loaded
- **THEN** the metadata panel shows camera model, lens, aperture, shutter speed, ISO, focal length, and date taken in a readable summary format

#### Scenario: Image without EXIF data
- **WHEN** a PNG image with no EXIF metadata is loaded
- **THEN** the EXIF camera data section is hidden or displays "No EXIF data"

### Requirement: Raw metadata tag dump
The metadata panel SHALL provide a scrollable section listing all raw EXIF, XMP, and IPTC tags as key-value pairs. This section is below the summary sections.

#### Scenario: View all metadata tags
- **WHEN** the user scrolls to the raw metadata section
- **THEN** all parsed metadata tags are listed with their tag names and values

#### Scenario: Large number of tags
- **WHEN** an image contains 100+ metadata tags
- **THEN** the raw metadata section is scrollable and does not affect the layout of the summary sections above

### Requirement: Metadata updates on navigation
The metadata panel SHALL update its content when the user navigates to a different image.

#### Scenario: Navigate to next image
- **WHEN** the user navigates to the next image while the metadata panel is visible
- **THEN** the metadata panel updates to show the new image's metadata
