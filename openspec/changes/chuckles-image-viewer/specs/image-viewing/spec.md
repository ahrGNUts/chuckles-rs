## ADDED Requirements

### Requirement: Display image at fit-to-window zoom
The application SHALL display the loaded image scaled to fit within the available canvas area while maintaining aspect ratio. This is the default zoom mode on image load.

#### Scenario: Image larger than window
- **WHEN** an image with dimensions larger than the canvas is loaded
- **THEN** the image is scaled down to fit entirely within the canvas, maintaining aspect ratio

#### Scenario: Image smaller than window
- **WHEN** an image with dimensions smaller than the canvas is loaded
- **THEN** the image is displayed at its actual size, centered in the canvas

### Requirement: Display image at actual size
The application SHALL display the loaded image at 100% zoom (1 image pixel = 1 screen pixel) when the user activates actual-size mode.

#### Scenario: Toggle to actual size
- **WHEN** the user presses the `1` key
- **THEN** the image is displayed at 100% zoom, centered on the previously visible center point

### Requirement: Zoom in and out
The application SHALL allow the user to zoom in and out of the displayed image using keyboard shortcuts and Ctrl+scroll.

#### Scenario: Zoom in with keyboard
- **WHEN** the user presses the `+` key
- **THEN** the image zoom level increases by one step, centered on the canvas center

#### Scenario: Zoom out with keyboard
- **WHEN** the user presses the `-` key
- **THEN** the image zoom level decreases by one step, centered on the canvas center

#### Scenario: Zoom with Ctrl+scroll
- **WHEN** the user holds Ctrl and scrolls the mouse wheel
- **THEN** the image zoom level changes, centered on the mouse cursor position, regardless of the configured scroll wheel mode

### Requirement: Fit to window
The application SHALL return to fit-to-window zoom when the user activates fit mode.

#### Scenario: Return to fit
- **WHEN** the user presses the `F` key
- **THEN** the image is scaled to fit the canvas, maintaining aspect ratio

### Requirement: Pan zoomed images
The application SHALL allow the user to pan a zoomed-in image by click-and-drag.

#### Scenario: Pan with mouse drag
- **WHEN** the image is zoomed beyond the canvas bounds and the user clicks and drags
- **THEN** the visible portion of the image moves with the drag direction
