use std::path::Path;

use crate::formats::ImageFormat;

/// Basic file information for display in the metadata panel.
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub filename: String,
    pub path: String,
    pub file_size: u64,
    pub width: u32,
    pub height: u32,
    pub color_depth: u32,
    pub format: ImageFormat,
}

impl FileInfo {
    pub fn from_path(
        path: &Path,
        width: u32,
        height: u32,
        color_depth: u32,
        format: ImageFormat,
    ) -> Option<Self> {
        let meta = std::fs::metadata(path).ok()?;
        Some(Self {
            filename: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            path: path.display().to_string(),
            file_size: meta.len(),
            width,
            height,
            color_depth,
            format,
        })
    }

    /// Human-readable file size string.
    pub fn formatted_size(&self) -> String {
        format_file_size(self.file_size)
    }

    /// Dimensions as "WxH" string.
    pub fn formatted_dimensions(&self) -> String {
        format!("{} x {}", self.width, self.height)
    }
}

fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_file_size_bytes() {
        assert_eq!(format_file_size(500), "500 B");
    }

    #[test]
    fn format_file_size_kilobytes() {
        assert_eq!(format_file_size(2048), "2.0 KB");
    }

    #[test]
    fn format_file_size_megabytes() {
        assert_eq!(format_file_size(4_500_000), "4.3 MB");
    }

    #[test]
    fn format_file_size_gigabytes() {
        assert_eq!(format_file_size(2_500_000_000), "2.3 GB");
    }

    #[test]
    fn formatted_dimensions_produces_expected_string() {
        let info = FileInfo {
            filename: "test.jpg".to_string(),
            path: "/tmp/test.jpg".to_string(),
            file_size: 1024,
            width: 4032,
            height: 3024,
            color_depth: 24,
            format: ImageFormat::Jpeg,
        };

        assert_eq!(info.formatted_dimensions(), "4032 x 3024");
    }
}
