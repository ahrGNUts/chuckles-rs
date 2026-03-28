## ADDED Requirements

### Requirement: Navigate to next image
The application SHALL display the next image in the sorted directory list when the user navigates forward.

#### Scenario: Next image via right arrow
- **WHEN** the user presses the Right arrow key
- **THEN** the next image in the sorted file list is displayed

#### Scenario: At last image
- **WHEN** the user is viewing the last image in the directory and presses Right arrow
- **THEN** the application remains on the current image (no wrap-around)

### Requirement: Navigate to previous image
The application SHALL display the previous image in the sorted directory list when the user navigates backward.

#### Scenario: Previous image via left arrow
- **WHEN** the user presses the Left arrow key
- **THEN** the previous image in the sorted file list is displayed

#### Scenario: At first image
- **WHEN** the user is viewing the first image in the directory and presses Left arrow
- **THEN** the application remains on the current image (no wrap-around)

### Requirement: Navigate to first and last image
The application SHALL jump to the first or last image in the sorted directory list.

#### Scenario: Jump to first image
- **WHEN** the user presses the Home key
- **THEN** the first image in the sorted file list is displayed

#### Scenario: Jump to last image
- **WHEN** the user presses the End key
- **THEN** the last image in the sorted file list is displayed

### Requirement: Natural filename sort as default
The application SHALL sort directory contents using natural sort order by default. Natural sort orders numeric sequences by their numeric value (img1, img2, img10) rather than lexicographically (img1, img10, img2).

#### Scenario: Directory with numbered files
- **WHEN** a directory contains files named photo1.jpg, photo2.jpg, photo10.jpg, photo20.jpg
- **THEN** the navigation order is photo1.jpg, photo2.jpg, photo10.jpg, photo20.jpg

### Requirement: Configurable sort modes
The application SHALL support sorting directory contents by: name (natural sort), date modified, file size, file type, and image dimensions. Each sort mode supports ascending and descending order.

#### Scenario: Sort by date modified
- **WHEN** the user selects "Date Modified" sort mode
- **THEN** images are ordered by their filesystem modification timestamp

#### Scenario: Sort by dimensions
- **WHEN** the user selects "Dimensions" sort mode
- **THEN** images are ordered by total pixel count (width * height), read from file headers without full image decoding using the `imagesize` crate. For formats not supported by `imagesize` (JXL, SVG), dimension reading falls back to the format-specific decoder. Files whose dimensions cannot be read are sorted to the end.

#### Scenario: Toggle sort direction
- **WHEN** the user toggles the sort direction
- **THEN** the current sort order reverses and the currently viewed image remains displayed

### Requirement: Scroll wheel navigates between images
By default, the unmodified mouse scroll wheel SHALL navigate between images (scroll down = next, scroll up = previous). This behavior is configurable to zoom instead. Ctrl+scroll always zooms regardless of this setting (see image-viewing spec).

#### Scenario: Scroll wheel default behavior
- **WHEN** the user scrolls the mouse wheel down without modifier keys
- **THEN** the next image in the sorted file list is displayed

#### Scenario: Scroll wheel configured to zoom
- **WHEN** the user has configured scroll wheel to zoom mode
- **THEN** scrolling the mouse wheel zooms in/out instead of navigating

### Requirement: Filter to supported formats only
The application SHALL include only files with supported image format extensions when scanning a directory. Non-image files, unsupported formats, and files without extensions are excluded from navigation. Format detection by file content (magic bytes) applies only during decoding of files that passed the extension filter.

#### Scenario: Mixed file types in directory
- **WHEN** a directory contains photo.jpg, readme.txt, image.png, data.csv
- **THEN** only photo.jpg and image.png appear in the navigation list

#### Scenario: File without extension
- **WHEN** a directory contains a file named `photo` with no extension that contains valid JPEG data
- **THEN** the file is excluded from the navigation list
