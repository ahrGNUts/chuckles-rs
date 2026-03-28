## ADDED Requirements

### Requirement: XDG-compliant config location
The application SHALL store its configuration file at `$XDG_CONFIG_HOME/chuckles/config.toml`, defaulting to `~/.config/chuckles/config.toml` if `XDG_CONFIG_HOME` is not set.

#### Scenario: Config file location
- **WHEN** the application starts and `XDG_CONFIG_HOME` is not set
- **THEN** configuration is read from `~/.config/chuckles/config.toml` if it exists

#### Scenario: Custom XDG_CONFIG_HOME
- **WHEN** the application starts and `XDG_CONFIG_HOME` is set to `/custom/config`
- **THEN** configuration is read from `/custom/config/chuckles/config.toml`

### Requirement: Sensible defaults without config file
The application SHALL function with all default settings when no configuration file exists. The application MUST NOT create a config file automatically.

#### Scenario: No config file present
- **WHEN** the application starts with no config file
- **THEN** all settings use defaults: natural sort ascending, scroll wheel navigates, fit-to-window zoom, sidebar hidden, thumbnail strip hidden

### Requirement: Configurable sort preferences
The configuration file SHALL support setting the default sort mode (name, date_modified, file_size, file_type, dimensions) and sort direction (ascending, descending).

#### Scenario: Custom default sort
- **WHEN** the config file contains `sort_mode = "date_modified"` and `sort_direction = "descending"`
- **THEN** directories are sorted by modification date in descending order by default

### Requirement: Configurable scroll wheel behavior
The configuration file SHALL support setting the mouse scroll wheel behavior to either "navigate" (prev/next image) or "zoom" (zoom in/out).

#### Scenario: Scroll wheel set to zoom
- **WHEN** the config file contains `scroll_wheel = "zoom"`
- **THEN** the mouse scroll wheel zooms in/out instead of navigating between images

### Requirement: Configurable window state persistence
The configuration file SHALL store the last window size, position, sidebar visibility, and thumbnail strip visibility. On startup, persisted window state in the config file takes precedence over hard-coded defaults. Hard-coded defaults apply only when no config file exists (first-ever launch).

#### Scenario: Window state saved on close
- **WHEN** the user closes the application and a config file exists
- **THEN** the current window dimensions, position, sidebar state, and thumbnail strip state are written to the config file

#### Scenario: First launch with no config file
- **WHEN** the application starts with no config file
- **THEN** hard-coded defaults are used (sidebar hidden, thumbnail strip hidden, default window size)

#### Scenario: Subsequent launch with config file
- **WHEN** the application starts and a config file exists with persisted window state
- **THEN** the persisted window state is restored (size, position, sidebar visibility, thumbnail strip visibility)

### Requirement: Configurable default zoom mode
The configuration file SHALL support setting the default zoom mode when opening an image to either "fit" (fit to window) or "actual" (100% / actual size).

#### Scenario: Default zoom set to actual size
- **WHEN** the config file contains `default_zoom = "actual"`
- **THEN** newly opened images are displayed at 100% zoom instead of fit-to-window

#### Scenario: Default zoom not set
- **WHEN** the config file has no `default_zoom` setting
- **THEN** newly opened images use fit-to-window zoom (the default)

### Requirement: Configurable background color
The configuration file SHALL support setting the background color of the image canvas.

#### Scenario: Custom background color
- **WHEN** the config file contains `background_color = "#1a1a2e"`
- **THEN** the image canvas background is rendered with that color
