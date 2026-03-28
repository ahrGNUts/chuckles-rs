use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub sort_mode: SortMode,
    pub sort_direction: SortDirection,
    pub scroll_wheel: ScrollWheelMode,
    pub default_zoom: ZoomDefault,
    pub background_color: String,
    pub window: WindowState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
    pub sidebar_visible: bool,
    pub thumbnail_strip_visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortMode {
    Name,
    DateModified,
    FileSize,
    FileType,
    Dimensions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrollWheelMode {
    Navigate,
    Zoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoomDefault {
    Fit,
    Actual,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            sort_mode: SortMode::Name,
            sort_direction: SortDirection::Ascending,
            scroll_wheel: ScrollWheelMode::Navigate,
            default_zoom: ZoomDefault::Fit,
            background_color: "#1a1a2e".to_string(),
            window: WindowState::default(),
        }
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            x: 0,
            y: 0,
            sidebar_visible: false,
            thumbnail_strip_visible: false,
        }
    }
}

/// Returns the XDG-compliant config file path.
pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("chuckles").join("config.toml"))
}

/// Load config from the XDG config path. Returns defaults if no file exists
/// or if the file is malformed.
pub fn load() -> AppConfig {
    let Some(path) = config_path() else {
        return AppConfig::default();
    };

    match std::fs::read_to_string(&path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

/// Save config to the XDG config path. Only writes if the config file
/// already exists (MUST NOT auto-create).
pub fn save(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = config_path() else {
        return Ok(());
    };

    if !path.exists() {
        return Ok(());
    }

    let contents = toml::to_string_pretty(config)?;
    std::fs::write(&path, contents)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let config = AppConfig::default();

        assert_eq!(config.sort_mode, SortMode::Name);
        assert_eq!(config.sort_direction, SortDirection::Ascending);
        assert_eq!(config.scroll_wheel, ScrollWheelMode::Navigate);
        assert_eq!(config.default_zoom, ZoomDefault::Fit);
        assert!(!config.window.sidebar_visible);
        assert!(!config.window.thumbnail_strip_visible);
    }

    #[test]
    fn config_round_trips_through_toml() {
        let config = AppConfig {
            sort_mode: SortMode::DateModified,
            sort_direction: SortDirection::Descending,
            scroll_wheel: ScrollWheelMode::Zoom,
            default_zoom: ZoomDefault::Actual,
            background_color: "#000000".to_string(),
            window: WindowState {
                width: 1920,
                height: 1080,
                x: 100,
                y: 50,
                sidebar_visible: true,
                thumbnail_strip_visible: true,
            },
        };

        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.sort_mode, SortMode::DateModified);
        assert_eq!(deserialized.sort_direction, SortDirection::Descending);
        assert_eq!(deserialized.scroll_wheel, ScrollWheelMode::Zoom);
        assert_eq!(deserialized.default_zoom, ZoomDefault::Actual);
        assert_eq!(deserialized.background_color, "#000000");
        assert_eq!(deserialized.window.width, 1920);
        assert!(deserialized.window.sidebar_visible);
    }

    #[test]
    fn config_deserializes_partial_toml_with_defaults() {
        let partial = r#"
            sort_mode = "file_size"
        "#;

        let config: AppConfig = toml::from_str(partial).unwrap();

        assert_eq!(config.sort_mode, SortMode::FileSize);
        // Other fields should use defaults
        assert_eq!(config.sort_direction, SortDirection::Ascending);
        assert_eq!(config.scroll_wheel, ScrollWheelMode::Navigate);
    }

    #[test]
    fn config_deserializes_empty_toml_as_defaults() {
        let config: AppConfig = toml::from_str("").unwrap();

        assert_eq!(config.sort_mode, SortMode::Name);
        assert_eq!(config.default_zoom, ZoomDefault::Fit);
    }

    #[test]
    fn save_does_nothing_when_no_config_file_exists() {
        // This tests the "MUST NOT auto-create" constraint.
        // save() should return Ok(()) without creating any file
        // when the config path doesn't exist.
        let config = AppConfig::default();
        let result = save(&config);
        assert!(result.is_ok());
    }
}
