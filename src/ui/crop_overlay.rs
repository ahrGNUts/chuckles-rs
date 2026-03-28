use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::state::AppState;

/// Aspect ratio presets for cropping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CropAspect {
    Free,
    Original,
    Square,
    Ratio4x3,
    Ratio16x9,
    Ratio3x2,
}

/// State for the interactive crop overlay.
#[derive(Debug, Clone)]
pub struct CropState {
    /// Crop rectangle in image pixel coordinates.
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// Image dimensions.
    pub img_width: f64,
    pub img_height: f64,
    /// Current aspect ratio constraint.
    pub aspect: CropAspect,
    /// Whether the user is currently dragging.
    pub dragging: bool,
    pub drag_start: (f64, f64),
}

impl CropState {
    pub fn new(img_width: f64, img_height: f64) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: img_width,
            height: img_height,
            img_width,
            img_height,
            aspect: CropAspect::Free,
            dragging: false,
            drag_start: (0.0, 0.0),
        }
    }

    pub fn set_aspect(&mut self, aspect: CropAspect) {
        self.aspect = aspect;
        self.enforce_aspect();
    }

    fn enforce_aspect(&mut self) {
        let ratio = match self.aspect {
            CropAspect::Free => return,
            CropAspect::Original => self.img_width / self.img_height,
            CropAspect::Square => 1.0,
            CropAspect::Ratio4x3 => 4.0 / 3.0,
            CropAspect::Ratio16x9 => 16.0 / 9.0,
            CropAspect::Ratio3x2 => 3.0 / 2.0,
        };

        // Adjust height to match ratio, clamped to image bounds
        let new_h = (self.width / ratio).min(self.img_height - self.y);
        self.height = new_h;
    }

    /// Clamp the crop rectangle to image bounds.
    pub fn clamp_to_image(&mut self) {
        self.x = self.x.max(0.0).min(self.img_width - 1.0);
        self.y = self.y.max(0.0).min(self.img_height - 1.0);
        self.width = self.width.max(1.0).min(self.img_width - self.x);
        self.height = self.height.max(1.0).min(self.img_height - self.y);
    }

    /// Get the crop rectangle as integer pixel values.
    pub fn as_rect(&self) -> (u32, u32, u32, u32) {
        (
            self.x.round() as u32,
            self.y.round() as u32,
            self.width.round().max(1.0) as u32,
            self.height.round().max(1.0) as u32,
        )
    }
}

/// Draw the crop overlay on the canvas. Called from the canvas draw function
/// when crop mode is active.
pub fn draw_crop_overlay(
    cr: &gtk4::cairo::Context,
    crop: &CropState,
    scale: f64,
    offset_x: f64,
    offset_y: f64,
) {
    // Semi-transparent dark overlay outside the crop region
    cr.save().unwrap();

    // Draw the full canvas darkened
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
    let _ = cr.paint();

    // Cut out the crop region (clear it)
    let cx = offset_x + crop.x * scale;
    let cy = offset_y + crop.y * scale;
    let cw = crop.width * scale;
    let ch = crop.height * scale;

    // Re-draw the image in the crop region by clearing the dark overlay there
    cr.set_operator(gtk4::cairo::Operator::Clear);
    cr.rectangle(cx, cy, cw, ch);
    let _ = cr.fill();

    // Draw crop border
    cr.set_operator(gtk4::cairo::Operator::Over);
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.9);
    cr.set_line_width(2.0);
    cr.rectangle(cx, cy, cw, ch);
    let _ = cr.stroke();

    // Draw rule-of-thirds grid lines
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.3);
    cr.set_line_width(1.0);
    for i in 1..3 {
        let frac = i as f64 / 3.0;
        // Vertical lines
        let lx = cx + cw * frac;
        cr.move_to(lx, cy);
        cr.line_to(lx, cy + ch);
        let _ = cr.stroke();
        // Horizontal lines
        let ly = cy + ch * frac;
        cr.move_to(cx, ly);
        cr.line_to(cx + cw, ly);
        let _ = cr.stroke();
    }

    // Draw corner handles
    let handle_size = 8.0;
    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
    for &(hx, hy) in &[(cx, cy), (cx + cw, cy), (cx, cy + ch), (cx + cw, cy + ch)] {
        cr.rectangle(
            hx - handle_size / 2.0,
            hy - handle_size / 2.0,
            handle_size,
            handle_size,
        );
        let _ = cr.fill();
    }

    cr.restore().unwrap();
}

/// Apply the crop to the current image.
pub fn apply_crop(state: &Rc<RefCell<AppState>>) {
    let (x, y, w, h) = {
        let s = state.borrow();
        let Some(crop) = &s.crop_state else {
            return;
        };
        crop.as_rect()
    };

    let mut s = state.borrow_mut();
    let Some(pixbuf) = &s.current_pixbuf else {
        return;
    };
    let Some(bytes) = pixbuf.pixel_bytes() else {
        return;
    };
    let img_w = pixbuf.width() as u32;
    let img_h = pixbuf.height() as u32;

    if x + w > img_w || y + h > img_h || w == 0 || h == 0 {
        return;
    }

    let Some(rgba) = image::RgbaImage::from_raw(img_w, img_h, bytes.to_vec()) else {
        return;
    };
    let img = image::DynamicImage::ImageRgba8(rgba);
    let cropped = img.crop_imm(x, y, w, h);
    let cropped_rgba = cropped.to_rgba8();
    let (cw, ch) = (cropped_rgba.width(), cropped_rgba.height());
    let raw = cropped_rgba.into_raw();

    let (format, color_depth) = s
        .current_image
        .as_ref()
        .map(|img| (img.format, img.color_depth))
        .unwrap_or((crate::formats::ImageFormat::Png, 32));
    s.current_image = Some(crate::formats::DecodedImage {
        width: cw,
        height: ch,
        pixels: raw,
        format,
        color_depth,
    });

    let pixels = &s.current_image.as_ref().unwrap().pixels;
    let new_pixbuf = gtk4::gdk_pixbuf::Pixbuf::from_bytes(
        &gtk4::glib::Bytes::from(pixels.as_slice()),
        gtk4::gdk_pixbuf::Colorspace::Rgb,
        true,
        8,
        cw as i32,
        ch as i32,
        (cw * 4) as i32,
    );
    s.current_pixbuf = Some(new_pixbuf);
    s.has_unsaved_edits = true;
    s.crop_state = None;

    let cb = s.on_image_changed.clone();
    drop(s);
    if let Some(cb) = cb {
        cb();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_state_new_covers_full_image() {
        let crop = CropState::new(1920.0, 1080.0);

        assert_eq!(crop.x, 0.0);
        assert_eq!(crop.y, 0.0);
        assert_eq!(crop.width, 1920.0);
        assert_eq!(crop.height, 1080.0);
    }

    #[test]
    fn as_rect_rounds_to_integers() {
        let crop = CropState {
            x: 10.4,
            y: 20.6,
            width: 100.5,
            height: 200.3,
            img_width: 1920.0,
            img_height: 1080.0,
            aspect: CropAspect::Free,
            dragging: false,
            drag_start: (0.0, 0.0),
        };

        let (x, y, w, h) = crop.as_rect();

        assert_eq!(x, 10);
        assert_eq!(y, 21);
        assert_eq!(w, 101); // 100.5 rounds to 101
        assert_eq!(h, 200);
    }

    #[test]
    fn clamp_keeps_rect_within_image() {
        let mut crop = CropState::new(100.0, 100.0);
        crop.x = -10.0;
        crop.y = 90.0;
        crop.width = 200.0;
        crop.height = 50.0;

        crop.clamp_to_image();

        assert_eq!(crop.x, 0.0);
        assert!(crop.width <= 100.0);
        assert!(crop.y + crop.height <= 100.0);
    }

    #[test]
    fn set_aspect_square_adjusts_height() {
        let mut crop = CropState::new(200.0, 100.0);
        crop.width = 80.0;
        crop.height = 60.0;

        crop.set_aspect(CropAspect::Square);

        assert!((crop.height - 80.0).abs() < 0.01);
    }

    #[test]
    fn set_aspect_free_does_not_change_dimensions() {
        let mut crop = CropState::new(200.0, 100.0);
        crop.width = 80.0;
        crop.height = 60.0;

        crop.set_aspect(CropAspect::Free);

        assert_eq!(crop.width, 80.0);
        assert_eq!(crop.height, 60.0);
    }
}
