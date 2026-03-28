## ADDED Requirements

### Requirement: GTK4 header bar with image info and controls
The windowed mode SHALL display a GTK4 HeaderBar containing: the application name, the current filename, the image index and total count (e.g., "42 of 187"), sort mode selector, zoom controls (fit/actual size), and an edit mode toggle.

#### Scenario: Header bar displays image info
- **WHEN** an image is displayed in windowed mode
- **THEN** the header bar shows the filename, position in directory (e.g., "42 of 187"), and active sort mode

#### Scenario: Header bar before directory scan completes
- **WHEN** the image is displayed but the directory scan is still in progress
- **THEN** the header bar shows the filename without an index count until the scan completes

### Requirement: Central image canvas
The windowed mode SHALL display the image in a central canvas area that supports zoom and pan. The canvas fills all available space not occupied by the header bar, sidebar, or thumbnail strip.

#### Scenario: Canvas resizes with window
- **WHEN** the user resizes the application window
- **THEN** the image canvas resizes accordingly and the image is re-fitted if in fit-to-window mode

### Requirement: Toggleable metadata sidebar
The windowed mode SHALL provide a metadata sidebar on the right side of the window. The sidebar is toggled visible/hidden with the `I` key.

#### Scenario: Toggle sidebar on
- **WHEN** the user presses `I` and the sidebar is hidden
- **THEN** the metadata sidebar appears on the right side of the window

#### Scenario: Toggle sidebar off
- **WHEN** the user presses `I` and the sidebar is visible
- **THEN** the metadata sidebar hides and the image canvas expands to fill the space

### Requirement: Optional thumbnail strip
The windowed mode SHALL provide a horizontal thumbnail strip along the bottom of the window showing neighboring images in the directory. The strip is toggled with the `T` key.

#### Scenario: Thumbnail strip shows current context
- **WHEN** the thumbnail strip is visible
- **THEN** it displays thumbnails of neighboring images with the current image highlighted

#### Scenario: Click thumbnail to navigate
- **WHEN** the user clicks a thumbnail in the strip
- **THEN** the main canvas displays that image and the strip scrolls to keep it centered

#### Scenario: Lazy thumbnail loading
- **WHEN** the thumbnail strip is scrolled to reveal new thumbnails
- **THEN** thumbnails are generated on demand, not all at once

### Requirement: Toggleable edit tools panel
The windowed mode SHALL provide an edit tools panel that can be shown or hidden via the "Edit" button in the header bar. The panel displays buttons for: rotate left, rotate right, flip horizontal, flip vertical, crop, and resize. Keyboard shortcuts for these operations work regardless of whether the panel is visible. The panel provides mouse access to editing functions without requiring shortcut memorization.

#### Scenario: Show edit tools panel
- **WHEN** the user clicks the "Edit" button in the header bar and the panel is hidden
- **THEN** an edit tools panel appears with buttons for all geometric editing operations

#### Scenario: Hide edit tools panel
- **WHEN** the user clicks the "Edit" button in the header bar and the panel is visible
- **THEN** the edit tools panel hides and the image canvas expands

#### Scenario: Keyboard shortcuts independent of panel
- **WHEN** the edit tools panel is hidden and the user presses `R`
- **THEN** the image rotates 90 degrees clockwise (shortcuts work regardless of panel visibility)

### Requirement: Navigation arrow overlays
The windowed mode SHALL display semi-transparent left and right arrow overlays on the image canvas edges when the mouse hovers over the canvas.

#### Scenario: Click arrow to navigate
- **WHEN** the user clicks the right arrow overlay
- **THEN** the next image in the sorted file list is displayed

### Requirement: Window state persistence
The application SHALL remember and restore the window size, position, sidebar visibility, and thumbnail strip visibility between sessions.

#### Scenario: Reopen after close
- **WHEN** the user closes chuckles and reopens it
- **THEN** the window appears at its previous size and position, with sidebar and thumbnail strip in their previous visibility state
