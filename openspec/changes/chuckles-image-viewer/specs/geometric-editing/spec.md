## ADDED Requirements

### Requirement: Rotate image 90 degrees
The application SHALL rotate the displayed image 90 degrees clockwise or counter-clockwise. The rotation is performed by decoding, transforming, and re-encoding the image at maximum quality. A future enhancement may add lossless JPEG rotation via DCT coefficient manipulation, pending availability of a mature pure-Rust library or acceptable C binding (e.g., libjpeg-turbo).

#### Scenario: Rotate clockwise
- **WHEN** the user presses the `R` key
- **THEN** the image rotates 90 degrees clockwise and the display updates immediately

#### Scenario: Rotate counter-clockwise
- **WHEN** the user presses the `L` key
- **THEN** the image rotates 90 degrees counter-clockwise and the display updates immediately

### Requirement: Flip image
The application SHALL flip the displayed image horizontally or vertically. Flipping is performed by decoding, transforming, and re-encoding at maximum quality. A future enhancement may add lossless JPEG flipping via DCT coefficient manipulation.

#### Scenario: Flip horizontal
- **WHEN** the user presses the `H` key
- **THEN** the image is mirrored along the vertical axis

#### Scenario: Flip vertical
- **WHEN** the user presses the `V` key
- **THEN** the image is mirrored along the horizontal axis

### Requirement: Interactive crop
The application SHALL provide an interactive crop mode with a draggable selection rectangle. The crop mode supports aspect ratio presets: free, original, 1:1, 4:3, 16:9, 3:2.

#### Scenario: Enter crop mode
- **WHEN** the user presses the `X` key
- **THEN** a crop overlay appears on the image with a draggable and resizable selection rectangle

#### Scenario: Crop with aspect ratio constraint
- **WHEN** the user selects a 16:9 aspect ratio preset in crop mode
- **THEN** the crop rectangle is constrained to 16:9 proportions

#### Scenario: Confirm crop
- **WHEN** the user presses Enter while in crop mode
- **THEN** the image is cropped to the selected region and displayed

#### Scenario: Cancel crop
- **WHEN** the user presses Escape while in crop mode
- **THEN** crop mode exits without modifying the image

### Requirement: JPEG geometric operations use maximum quality re-encoding
The application SHALL re-encode JPEG images at maximum quality when performing geometric operations (rotate, flip, crop, resize). Lossless JPEG operations via DCT coefficient manipulation are a planned future enhancement, pending availability of a mature pure-Rust library or acceptable C binding (e.g., libjpeg-turbo).

#### Scenario: JPEG rotation re-encoding
- **WHEN** the user performs any geometric operation on a JPEG image
- **THEN** the operation decodes, transforms, and re-encodes at maximum quality

### Requirement: Resize image
The application SHALL provide a resize dialog where the user can specify target dimensions or a percentage. Aspect ratio is locked by default.

#### Scenario: Resize by dimensions
- **WHEN** the user enters a width of 1920 in the resize dialog with aspect ratio locked
- **THEN** the height is automatically calculated to maintain the original aspect ratio

#### Scenario: Resize by percentage
- **WHEN** the user enters 50% in the resize dialog
- **THEN** both dimensions are halved

### Requirement: Save-As for all edits
All editing operations SHALL require Save-As to persist changes. The application MUST NOT silently overwrite the original file. Edits are applied in-memory and displayed immediately but not persisted until the user explicitly saves.

#### Scenario: Save edited image
- **WHEN** the user presses Ctrl+Shift+S after making edits
- **THEN** a Save-As file dialog appears with the original filename as the default

#### Scenario: Navigate away from edited image
- **WHEN** the user navigates to another image after making unsaved edits
- **THEN** the application prompts whether to save or discard changes
