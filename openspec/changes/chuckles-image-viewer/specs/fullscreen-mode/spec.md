## ADDED Requirements

### Requirement: Zero-chrome full-screen display
The full-screen mode SHALL display the image filling the entire screen with no visible UI chrome. All panels are hidden until the user hovers at a screen edge.

#### Scenario: Enter full-screen
- **WHEN** the user presses Enter or F11 in windowed mode
- **THEN** the window enters full-screen, the image fills the screen, and all UI elements are hidden

#### Scenario: Exit full-screen
- **WHEN** the user presses Escape in full-screen mode
- **THEN** the window returns to windowed mode at its previous size and position

#### Scenario: Enter and F11 are both toggles
- **WHEN** the user presses Enter or F11 in full-screen mode
- **THEN** the window returns to windowed mode

### Requirement: Fly-out thumbnail panel at bottom edge
A horizontal thumbnail strip SHALL slide in from the bottom of the screen when the mouse cursor enters the bottom edge detection zone. This matches the thumbnail strip position in windowed mode for consistency.

#### Scenario: Hover at bottom edge for thumbnails
- **WHEN** the user moves the mouse cursor to the bottom edge of the screen in full-screen mode
- **THEN** a thumbnail strip panel slides in from the bottom, showing neighboring images

#### Scenario: Leave bottom edge
- **WHEN** the user moves the mouse cursor away from the bottom panel
- **THEN** the thumbnail strip panel slides out and hides

### Requirement: Fly-out metadata panel at right edge
A metadata panel SHALL slide in from the right side of the screen when the mouse cursor enters the right edge detection zone. It displays the same metadata content as the windowed sidebar.

#### Scenario: Hover at right edge
- **WHEN** the user moves the mouse cursor to the right edge of the screen in full-screen mode
- **THEN** a metadata panel slides in from the right

### Requirement: Fly-out edit tools panel at left edge
An edit tools panel SHALL slide in from the left side of the screen when the mouse cursor enters the left edge detection zone. It provides access to rotate, flip, crop, and resize operations.

#### Scenario: Hover at left edge
- **WHEN** the user moves the mouse cursor to the left edge of the screen in full-screen mode
- **THEN** an edit tools panel slides in from the left with geometric editing options

### Requirement: Navigation controls integrated in thumbnail panel
The fly-out thumbnail panel at the bottom edge SHALL include navigation controls (prev/next buttons, zoom controls, sort mode selection) alongside the thumbnail strip. This consolidates bottom-edge functionality into a single panel.

#### Scenario: Bottom panel includes navigation
- **WHEN** the bottom fly-out panel is visible
- **THEN** it displays both the thumbnail strip and navigation/zoom/sort controls

### Requirement: Double-click enters full-screen
Double-clicking the image canvas in windowed mode SHALL enter full-screen mode. To exit full-screen, the user presses Escape (or Enter/F11 which act as toggles).

#### Scenario: Double-click in windowed mode
- **WHEN** the user double-clicks the image canvas in windowed mode
- **THEN** the application enters full-screen mode

#### Scenario: Double-click in full-screen mode
- **WHEN** the user double-clicks the image canvas in full-screen mode
- **THEN** nothing happens (use Escape to exit full-screen)
