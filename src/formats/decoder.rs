use std::path::Path;

use crate::formats::detection::{self, ImageFormat};

/// A decoded image ready for display. Pixels are always straight (non-premultiplied) RGBA,
/// 4 bytes per pixel, row-major.
#[derive(Debug)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub format: ImageFormat,
    /// Bits per pixel of the original source image (e.g., 24 for RGB, 32 for RGBA, 8 for grayscale).
    pub color_depth: u32,
}

/// Errors that can occur during image decoding.
#[derive(Debug)]
pub enum DecodeError {
    UnsupportedFormat(String),
    IoError(std::io::Error),
    DecodeFailure(String),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::UnsupportedFormat(path) => {
                write!(f, "Unsupported format: {path}")
            }
            DecodeError::IoError(err) => write!(f, "I/O error: {err}"),
            DecodeError::DecodeFailure(msg) => write!(f, "Decode error: {msg}"),
        }
    }
}

impl std::error::Error for DecodeError {}

impl From<std::io::Error> for DecodeError {
    fn from(err: std::io::Error) -> Self {
        DecodeError::IoError(err)
    }
}

impl From<image::ImageError> for DecodeError {
    fn from(err: image::ImageError) -> Self {
        DecodeError::DecodeFailure(err.to_string())
    }
}

/// Decode an image file, auto-detecting format by content then extension.
pub fn decode_file(path: &Path) -> Result<DecodedImage, DecodeError> {
    let format = detection::detect_format(path)
        .ok_or_else(|| DecodeError::UnsupportedFormat(path.display().to_string()))?;

    match format {
        ImageFormat::Svg => decode_svg(path, format),
        ImageFormat::Jxl => decode_jxl(path, format),
        _ => decode_with_image_crate(path, format),
    }
}

fn decode_with_image_crate(path: &Path, format: ImageFormat) -> Result<DecodedImage, DecodeError> {
    let img = image::open(path)?;
    let color_depth = match img.color() {
        image::ColorType::L8 => 8,
        image::ColorType::La8 => 16,
        image::ColorType::Rgb8 => 24,
        image::ColorType::Rgba8 => 32,
        image::ColorType::L16 => 16,
        image::ColorType::La16 => 32,
        image::ColorType::Rgb16 => 48,
        image::ColorType::Rgba16 => 64,
        image::ColorType::Rgb32F => 96,
        image::ColorType::Rgba32F => 128,
        _ => 24, // conservative default
    };
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    Ok(DecodedImage {
        width,
        height,
        pixels: rgba.into_raw(),
        format,
        color_depth,
    })
}

fn decode_jxl(path: &Path, format: ImageFormat) -> Result<DecodedImage, DecodeError> {
    let data = std::fs::read(path)?;
    let image = jxl_oxide::JxlImage::builder()
        .read(&*data)
        .map_err(|e| DecodeError::DecodeFailure(format!("JXL: {e}")))?;

    let render = image
        .render_frame(0)
        .map_err(|e| DecodeError::DecodeFailure(format!("JXL render: {e}")))?;

    let stream = render.image_all_channels();
    let width = stream.width() as u32;
    let height = stream.height() as u32;

    let channels = stream.channels();
    let color_depth = (channels as u32) * 8;
    let num_pixels = (width * height) as usize;
    let mut pixels = Vec::with_capacity(num_pixels * 4);

    let buf = stream.buf();

    for i in 0..num_pixels {
        if channels >= 3 {
            pixels.push(float_to_u8(buf[i]));
            pixels.push(float_to_u8(buf[num_pixels + i]));
            pixels.push(float_to_u8(buf[2 * num_pixels + i]));
            if channels >= 4 {
                pixels.push(float_to_u8(buf[3 * num_pixels + i]));
            } else {
                pixels.push(255);
            }
        } else {
            let v = float_to_u8(buf[i]);
            pixels.push(v);
            pixels.push(v);
            pixels.push(v);
            if channels >= 2 {
                pixels.push(float_to_u8(buf[num_pixels + i]));
            } else {
                pixels.push(255);
            }
        }
    }

    Ok(DecodedImage {
        width,
        height,
        pixels,
        format,
        color_depth,
    })
}

fn float_to_u8(val: f32) -> u8 {
    (val.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn decode_svg(path: &Path, format: ImageFormat) -> Result<DecodedImage, DecodeError> {
    let data = std::fs::read(path)?;
    let tree = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default())
        .map_err(|e| DecodeError::DecodeFailure(format!("SVG: {e}")))?;

    let size = tree.size();
    let width = size.width().ceil() as u32;
    let height = size.height().ceil() as u32;

    if width == 0 || height == 0 {
        return Err(DecodeError::DecodeFailure(
            "SVG has zero dimensions".to_string(),
        ));
    }

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| DecodeError::DecodeFailure("Failed to create pixmap for SVG".to_string()))?;

    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    // tiny-skia produces premultiplied RGBA. Convert to straight RGBA.
    let mut pixels = pixmap.data().to_vec();
    for chunk in pixels.chunks_exact_mut(4) {
        let a = chunk[3] as u16;
        if a > 0 && a < 255 {
            chunk[0] = ((chunk[0] as u16 * 255) / a).min(255) as u8;
            chunk[1] = ((chunk[1] as u16 * 255) / a).min(255) as u8;
            chunk[2] = ((chunk[2] as u16 * 255) / a).min(255) as u8;
        }
    }

    Ok(DecodedImage {
        width,
        height,
        pixels,
        format,
        color_depth: 32, // SVG is always rendered as RGBA
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_file_returns_error_for_unsupported_format() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.xyz");
        std::fs::write(&path, b"not an image").unwrap();

        let result = decode_file(&path);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DecodeError::UnsupportedFormat(_)
        ));
    }

    #[test]
    fn decode_file_returns_error_for_nonexistent_file() {
        let path = std::path::Path::new("/nonexistent/image.jpg");

        let result = decode_file(path);

        assert!(result.is_err());
    }

    #[test]
    fn decode_file_returns_error_for_corrupt_image() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("corrupt.png");
        // Write PNG magic bytes followed by garbage
        std::fs::write(
            &path,
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0xFF],
        )
        .unwrap();

        let result = decode_file(&path);

        assert!(result.is_err());
    }

    #[test]
    fn float_to_u8_clamps_values() {
        assert_eq!(float_to_u8(0.0), 0);
        assert_eq!(float_to_u8(1.0), 255);
        assert_eq!(float_to_u8(0.5), 128);
        assert_eq!(float_to_u8(-0.5), 0);
        assert_eq!(float_to_u8(1.5), 255);
    }
}
