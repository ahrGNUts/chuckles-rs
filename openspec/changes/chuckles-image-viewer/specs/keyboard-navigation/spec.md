## ADDED Requirements

### Requirement: Image navigation shortcuts
The application SHALL support keyboard shortcuts for image navigation: Left arrow (previous), Right arrow (next), Home (first), End (last).

#### Scenario: Navigate with arrow keys
- **WHEN** the user presses the Right arrow key
- **THEN** the next image in the sorted directory list is displayed

### Requirement: Zoom shortcuts
The application SHALL support keyboard shortcuts for zoom: `+` (zoom in), `-` (zoom out), `1` (actual size / 100%), `F` (fit to window).

#### Scenario: Zoom to actual size
- **WHEN** the user presses `1`
- **THEN** the image is displayed at 100% zoom

### Requirement: Mode toggle shortcuts
The application SHALL support Enter and F11 to toggle full-screen mode. Escape SHALL exit full-screen mode only (never close the application).

#### Scenario: Escape in full-screen
- **WHEN** the user presses Escape in full-screen mode
- **THEN** the application exits full-screen to windowed mode

#### Scenario: Escape in windowed mode
- **WHEN** the user presses Escape in windowed mode
- **THEN** nothing happens (the application does not close)

### Requirement: Editing shortcuts
The application SHALL support keyboard shortcuts for editing: `L` (rotate left), `R` (rotate right), `H` (flip horizontal), `V` (flip vertical), `X` (crop mode).

#### Scenario: Rotate with keyboard
- **WHEN** the user presses `R`
- **THEN** the image rotates 90 degrees clockwise

### Requirement: Panel toggle shortcuts
The application SHALL support `I` to toggle the metadata sidebar and `T` to toggle the thumbnail strip.

#### Scenario: Toggle metadata panel
- **WHEN** the user presses `I`
- **THEN** the metadata sidebar toggles between visible and hidden

### Requirement: Application quit shortcut
The application SHALL close when the user presses Ctrl+Q.

#### Scenario: Quit application
- **WHEN** the user presses Ctrl+Q
- **THEN** the application closes (prompting to save if there are unsaved edits)

### Requirement: Save As shortcut
The application SHALL open the Save-As dialog when the user presses Ctrl+Shift+S with unsaved edits. This follows the standard convention for Save-As across most desktop applications.

#### Scenario: Save As with unsaved edits
- **WHEN** the user presses Ctrl+Shift+S with pending edits
- **THEN** a Save-As file dialog opens

#### Scenario: Save As with no edits
- **WHEN** the user presses Ctrl+Shift+S with no pending edits
- **THEN** nothing happens
