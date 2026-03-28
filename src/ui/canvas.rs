use std::cell::RefCell;
use std::rc::Rc;

use gtk4::DrawingArea;
use gtk4::prelude::*;

use crate::ui::state::{AppState, ZoomMode};

/// Build the image canvas widget.
pub fn build_canvas(state: &Rc<RefCell<AppState>>) -> DrawingArea {
    let canvas = DrawingArea::new();
    canvas.set_hexpand(true);
    canvas.set_vexpand(true);

    let state = state.clone();
    canvas.set_draw_func(move |_area, cr, width, height| {
        let mut s = state.borrow_mut();

        // Background
        if let Ok(color) = parse_hex_color(&s.background_color) {
            cr.set_source_rgb(color.0, color.1, color.2);
        } else {
            cr.set_source_rgb(0.1, 0.1, 0.18);
        }
        let _ = cr.paint();

        let Some(pixbuf) = s.current_pixbuf.clone() else {
            return;
        };

        let img_w = pixbuf.width() as f64;
        let img_h = pixbuf.height() as f64;
        let canvas_w = width as f64;
        let canvas_h = height as f64;

        let scale = match s.zoom {
            ZoomMode::Fit => {
                let scale_x = canvas_w / img_w;
                let scale_y = canvas_h / img_h;
                let fit = scale_x.min(scale_y).min(1.0);
                // Store the computed fit scale so zoom_factor() returns the real value
                s.computed_fit_scale = fit;
                fit
            }
            ZoomMode::Actual => 1.0,
            ZoomMode::Custom(f) => f,
        };

        let scaled_w = img_w * scale;
        let scaled_h = img_h * scale;

        // Center the image in the canvas, then apply pan offset
        let x = (canvas_w - scaled_w) / 2.0 + s.pan_offset.0;
        let y = (canvas_h - scaled_h) / 2.0 + s.pan_offset.1;

        cr.save().unwrap();
        cr.translate(x, y);
        cr.scale(scale, scale);

        gtk4::gdk::prelude::GdkCairoContextExt::set_source_pixbuf(cr, &pixbuf, 0.0, 0.0);

        // Use appropriate interpolation based on zoom level
        if scale < 1.0 {
            cr.source().set_filter(gtk4::cairo::Filter::Bilinear);
        } else {
            cr.source().set_filter(gtk4::cairo::Filter::Nearest);
        }

        let _ = cr.paint();
        cr.restore().unwrap();
    });

    canvas
}

fn parse_hex_color(hex: &str) -> Result<(f64, f64, f64), ()> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err(());
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())? as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())? as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())? as f64 / 255.0;
    Ok((r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_color_valid() {
        let (r, g, b) = parse_hex_color("#1a1a2e").unwrap();
        assert!((r - 0.102).abs() < 0.01);
        assert!((g - 0.102).abs() < 0.01);
        assert!((b - 0.180).abs() < 0.01);
    }

    #[test]
    fn parse_hex_color_without_hash() {
        let (r, g, b) = parse_hex_color("ffffff").unwrap();
        assert!((r - 1.0).abs() < 0.01);
        assert!((g - 1.0).abs() < 0.01);
        assert!((b - 1.0).abs() < 0.01);
    }

    #[test]
    fn parse_hex_color_invalid() {
        assert!(parse_hex_color("xyz").is_err());
        assert!(parse_hex_color("#12").is_err());
    }
}
