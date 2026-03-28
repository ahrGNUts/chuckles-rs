use std::io::Read;
use std::path::Path;

/// Supported image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Gif,
    Bmp,
    WebP,
    Tiff,
    Jxl,
    Svg,
}

/// File extensions recognized during directory scanning.
const SUPPORTED_EXTENSIONS: &[(&str, ImageFormat)] = &[
    ("jpg", ImageFormat::Jpeg),
    ("jpeg", ImageFormat::Jpeg),
    ("jpe", ImageFormat::Jpeg),
    ("png", ImageFormat::Png),
    ("gif", ImageFormat::Gif),
    ("bmp", ImageFormat::Bmp),
    ("dib", ImageFormat::Bmp),
    ("webp", ImageFormat::WebP),
    ("tif", ImageFormat::Tiff),
    ("tiff", ImageFormat::Tiff),
    ("jxl", ImageFormat::Jxl),
    ("svg", ImageFormat::Svg),
    ("svgz", ImageFormat::Svg),
];

/// Check if a file extension is a supported image format.
pub fn is_supported_extension(path: &Path) -> bool {
    extension_format(path).is_some()
}

/// Get the format hint from a file extension.
pub fn extension_format(path: &Path) -> Option<ImageFormat> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    SUPPORTED_EXTENSIONS
        .iter()
        .find(|(e, _)| *e == ext)
        .map(|(_, fmt)| *fmt)
}

/// Detect image format from file content (magic bytes).
/// Falls back to extension-based detection if magic bytes are inconclusive.
pub fn detect_format(path: &Path) -> Option<ImageFormat> {
    if let Some(fmt) = detect_by_magic_bytes(path) {
        return Some(fmt);
    }
    extension_format(path)
}

fn detect_by_magic_bytes(path: &Path) -> Option<ImageFormat> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut header = [0u8; 64];
    let bytes_read = file.read(&mut header).ok()?;
    if bytes_read < 4 {
        return None;
    }

    let buf = &header[..bytes_read];

    // JPEG: FF D8 FF
    if buf.len() >= 3 && buf[0] == 0xFF && buf[1] == 0xD8 && buf[2] == 0xFF {
        return Some(ImageFormat::Jpeg);
    }

    // PNG: 89 50 4E 47
    if buf.len() >= 4 && buf[..4] == [0x89, 0x50, 0x4E, 0x47] {
        return Some(ImageFormat::Png);
    }

    // GIF: GIF87a or GIF89a
    if buf.len() >= 6 && &buf[..3] == b"GIF" {
        return Some(ImageFormat::Gif);
    }

    // BMP: BM
    if buf.len() >= 2 && buf[0] == b'B' && buf[1] == b'M' {
        return Some(ImageFormat::Bmp);
    }

    // WebP: RIFF....WEBP
    if buf.len() >= 12 && &buf[..4] == b"RIFF" && &buf[8..12] == b"WEBP" {
        return Some(ImageFormat::WebP);
    }

    // TIFF: II (little-endian) or MM (big-endian)
    if buf.len() >= 4
        && ((buf[0] == b'I' && buf[1] == b'I' && buf[2] == 42 && buf[3] == 0)
            || (buf[0] == b'M' && buf[1] == b'M' && buf[2] == 0 && buf[3] == 42))
    {
        return Some(ImageFormat::Tiff);
    }

    // JPEG XL: FF 0A (naked codestream) or 00 00 00 0C 4A 58 4C 20 (container)
    if buf.len() >= 2 && buf[0] == 0xFF && buf[1] == 0x0A {
        return Some(ImageFormat::Jxl);
    }
    if buf.len() >= 12 && buf[..4] == [0x00, 0x00, 0x00, 0x0C] && &buf[4..8] == b"JXL " {
        return Some(ImageFormat::Jxl);
    }

    // SVG: starts with <?xml or <svg (possibly with BOM or whitespace)
    if let Ok(text) = std::str::from_utf8(buf) {
        let trimmed = text.trim_start();
        if trimmed.starts_with("<?xml") || trimmed.starts_with("<svg") {
            return Some(ImageFormat::Svg);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_extension_recognizes_common_formats() {
        assert!(is_supported_extension(Path::new("photo.jpg")));
        assert!(is_supported_extension(Path::new("photo.JPEG")));
        assert!(is_supported_extension(Path::new("image.png")));
        assert!(is_supported_extension(Path::new("anim.gif")));
        assert!(is_supported_extension(Path::new("photo.webp")));
        assert!(is_supported_extension(Path::new("photo.jxl")));
        assert!(is_supported_extension(Path::new("drawing.svg")));
    }

    #[test]
    fn supported_extension_rejects_unsupported() {
        assert!(!is_supported_extension(Path::new("readme.txt")));
        assert!(!is_supported_extension(Path::new("data.csv")));
        assert!(!is_supported_extension(Path::new("photo.heic")));
        assert!(!is_supported_extension(Path::new("photo.raw")));
    }

    #[test]
    fn supported_extension_rejects_no_extension() {
        assert!(!is_supported_extension(Path::new("photo")));
        assert!(!is_supported_extension(Path::new(".hidden")));
    }

    #[test]
    fn extension_format_is_case_insensitive() {
        assert_eq!(
            extension_format(Path::new("PHOTO.JPG")),
            Some(ImageFormat::Jpeg)
        );
        assert_eq!(
            extension_format(Path::new("image.Png")),
            Some(ImageFormat::Png)
        );
    }

    #[test]
    fn detect_jpeg_magic_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.dat");
        // JPEG starts with FF D8 FF
        std::fs::write(&path, &[0xFF, 0xD8, 0xFF, 0xE0, 0x00]).unwrap();
        assert_eq!(detect_by_magic_bytes(&path), Some(ImageFormat::Jpeg));
    }

    #[test]
    fn detect_png_magic_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.dat");
        std::fs::write(&path, &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A]).unwrap();
        assert_eq!(detect_by_magic_bytes(&path), Some(ImageFormat::Png));
    }

    #[test]
    fn detect_gif_magic_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.dat");
        std::fs::write(&path, b"GIF89a\x00\x00").unwrap();
        assert_eq!(detect_by_magic_bytes(&path), Some(ImageFormat::Gif));
    }

    #[test]
    fn detect_bmp_magic_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.dat");
        std::fs::write(&path, b"BM\x00\x00\x00\x00").unwrap();
        assert_eq!(detect_by_magic_bytes(&path), Some(ImageFormat::Bmp));
    }

    #[test]
    fn detect_webp_magic_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.dat");
        std::fs::write(&path, b"RIFF\x00\x00\x00\x00WEBP").unwrap();
        assert_eq!(detect_by_magic_bytes(&path), Some(ImageFormat::WebP));
    }

    #[test]
    fn detect_tiff_little_endian_magic_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.dat");
        std::fs::write(&path, &[b'I', b'I', 42, 0, 0, 0]).unwrap();
        assert_eq!(detect_by_magic_bytes(&path), Some(ImageFormat::Tiff));
    }

    #[test]
    fn detect_svg_magic_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.dat");
        std::fs::write(&path, b"<?xml version=").unwrap();
        assert_eq!(detect_by_magic_bytes(&path), Some(ImageFormat::Svg));
    }

    #[test]
    fn detect_format_falls_back_to_extension() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.jpg");
        // Write garbage that doesn't match any magic bytes
        std::fs::write(&path, b"not a real image").unwrap();
        assert_eq!(detect_format(&path), Some(ImageFormat::Jpeg));
    }

    #[test]
    fn detect_format_mismatched_extension_prefers_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("photo.jpg");
        // Write PNG magic bytes in a .jpg file
        std::fs::write(&path, &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A]).unwrap();
        assert_eq!(detect_format(&path), Some(ImageFormat::Png));
    }
}
