mod decoder;
mod detection;

pub use decoder::{DecodeError, DecodedImage, decode_file};
pub use detection::{ImageFormat, detect_format, is_supported_extension};
