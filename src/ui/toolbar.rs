use std::cell::RefCell;
use std::rc::Rc;

use gtk4::Button;
use gtk4::prelude::*;

use crate::editing;
use crate::ui::state::AppState;

/// Build the edit tools panel with buttons for geometric operations.
pub fn build_edit_toolbar(state: &Rc<RefCell<AppState>>) -> gtk4::Box {
    let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    toolbar.set_margin_top(4);
    toolbar.set_margin_bottom(4);
    toolbar.set_margin_start(8);
    toolbar.set_margin_end(8);
    toolbar.set_halign(gtk4::Align::Center);

    let buttons = [
        ("Rotate Left", EditAction::RotateLeft),
        ("Rotate Right", EditAction::RotateRight),
        ("Flip H", EditAction::FlipHorizontal),
        ("Flip V", EditAction::FlipVertical),
        ("Crop", EditAction::Crop),
        ("Resize", EditAction::Resize),
    ];

    for (label, action) in buttons {
        let btn = Button::with_label(label);
        let state = state.clone();
        btn.connect_clicked(move |_| {
            apply_edit_action(&state, action);
        });
        toolbar.append(&btn);
    }

    toolbar
}

#[derive(Debug, Clone, Copy)]
pub enum EditAction {
    RotateLeft,
    RotateRight,
    FlipHorizontal,
    FlipVertical,
    Crop,
    Resize,
}

pub fn apply_edit_action(state: &Rc<RefCell<AppState>>, action: EditAction) {
    // Crop and Resize need dialog-based interaction, handled separately.
    match action {
        EditAction::Crop | EditAction::Resize => {
            // TODO: implement crop/resize dialogs
            return;
        }
        _ => {}
    }

    let mut s = state.borrow_mut();
    let Some(pixbuf) = &s.current_pixbuf else {
        return;
    };

    // Convert pixbuf to DynamicImage for editing
    let Some(bytes) = pixbuf.pixel_bytes() else {
        return;
    };
    let width = pixbuf.width() as u32;
    let height = pixbuf.height() as u32;
    let has_alpha = pixbuf.has_alpha();

    let img = if has_alpha {
        let Some(buf) = image::RgbaImage::from_raw(width, height, bytes.to_vec()) else {
            return;
        };
        image::DynamicImage::ImageRgba8(buf)
    } else {
        let Some(buf) = image::RgbImage::from_raw(width, height, bytes.to_vec()) else {
            return;
        };
        image::DynamicImage::ImageRgb8(buf)
    };

    let result = match action {
        EditAction::RotateRight => editing::rotate_cw(&img),
        EditAction::RotateLeft => editing::rotate_ccw(&img),
        EditAction::FlipHorizontal => editing::flip_horizontal(&img),
        EditAction::FlipVertical => editing::flip_vertical(&img),
        _ => unreachable!(),
    };

    // Convert back to pixbuf
    let rgba = result.to_rgba8();
    let (new_w, new_h) = (rgba.width(), rgba.height());
    let raw = rgba.into_raw();

    let new_pixbuf = gtk4::gdk_pixbuf::Pixbuf::from_bytes(
        &gtk4::glib::Bytes::from(&raw),
        gtk4::gdk_pixbuf::Colorspace::Rgb,
        true,
        8,
        new_w as i32,
        new_h as i32,
        (new_w * 4) as i32,
    );

    // Update both pixbuf and DecodedImage to keep dimensions/pixels in sync.
    let format = s
        .current_image
        .as_ref()
        .map(|img| img.format)
        .unwrap_or(crate::formats::ImageFormat::Png);
    s.current_image = Some(crate::formats::DecodedImage {
        width: new_w,
        height: new_h,
        pixels: raw.clone(),
        format,
    });
    s.current_pixbuf = Some(new_pixbuf);
    s.has_unsaved_edits = true;

    let cb = s.on_image_changed.clone();
    drop(s);
    if let Some(cb) = cb {
        cb();
    }
}
