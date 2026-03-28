use image::{DynamicImage, imageops::FilterType};

/// Rotate an image 90 degrees clockwise.
pub fn rotate_cw(img: &DynamicImage) -> DynamicImage {
    img.rotate90()
}

/// Rotate an image 90 degrees counter-clockwise.
pub fn rotate_ccw(img: &DynamicImage) -> DynamicImage {
    img.rotate270()
}

/// Flip an image horizontally (mirror along vertical axis).
pub fn flip_horizontal(img: &DynamicImage) -> DynamicImage {
    img.fliph()
}

/// Flip an image vertically (mirror along horizontal axis).
pub fn flip_vertical(img: &DynamicImage) -> DynamicImage {
    img.flipv()
}

/// Resize an image to the given dimensions using Lanczos3 resampling.
/// Aspect ratio is NOT enforced here -- caller should compute correct dimensions.
pub fn resize(img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    img.resize_exact(width, height, FilterType::Lanczos3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbaImage};

    fn make_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::new(width, height))
    }

    #[test]
    fn rotate_cw_swaps_dimensions() {
        let img = make_test_image(200, 100);

        let rotated = rotate_cw(&img);

        assert_eq!(rotated.width(), 100);
        assert_eq!(rotated.height(), 200);
    }

    #[test]
    fn rotate_ccw_swaps_dimensions() {
        let img = make_test_image(200, 100);

        let rotated = rotate_ccw(&img);

        assert_eq!(rotated.width(), 100);
        assert_eq!(rotated.height(), 200);
    }

    #[test]
    fn flip_horizontal_preserves_dimensions() {
        let img = make_test_image(200, 100);

        let flipped = flip_horizontal(&img);

        assert_eq!(flipped.width(), 200);
        assert_eq!(flipped.height(), 100);
    }

    #[test]
    fn flip_vertical_preserves_dimensions() {
        let img = make_test_image(200, 100);

        let flipped = flip_vertical(&img);

        assert_eq!(flipped.width(), 200);
        assert_eq!(flipped.height(), 100);
    }

    #[test]
    fn resize_produces_exact_dimensions() {
        let img = make_test_image(200, 100);

        let resized = resize(&img, 50, 25);

        assert_eq!(resized.width(), 50);
        assert_eq!(resized.height(), 25);
    }

    #[test]
    fn rotate_cw_four_times_returns_to_original_dimensions() {
        let img = make_test_image(300, 200);

        let result = rotate_cw(&rotate_cw(&rotate_cw(&rotate_cw(&img))));

        assert_eq!(result.width(), 300);
        assert_eq!(result.height(), 200);
    }
}
