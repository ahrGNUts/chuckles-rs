use std::cell::RefCell;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Button, DropDown, Label, SpinButton, StringList};

use crate::editing;
use crate::ui::state::AppState;

/// Show the resize dialog.
pub fn show_resize_dialog(window: &ApplicationWindow, state: &Rc<RefCell<AppState>>) {
    let s = state.borrow();
    let Some(pixbuf) = &s.current_pixbuf else {
        return;
    };
    let orig_w = pixbuf.width() as f64;
    let orig_h = pixbuf.height() as f64;
    drop(s);

    let dialog = gtk4::Window::builder()
        .title("Resize Image")
        .transient_for(window)
        .modal(true)
        .default_width(300)
        .default_height(200)
        .build();

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_margin_start(16);
    content.set_margin_end(16);

    // Width/Height inputs
    let grid = gtk4::Grid::new();
    grid.set_row_spacing(8);
    grid.set_column_spacing(8);

    grid.attach(&Label::new(Some("Width:")), 0, 0, 1, 1);
    let width_spin = SpinButton::with_range(1.0, 99999.0, 1.0);
    width_spin.set_value(orig_w);
    grid.attach(&width_spin, 1, 0, 1, 1);

    grid.attach(&Label::new(Some("Height:")), 0, 1, 1, 1);
    let height_spin = SpinButton::with_range(1.0, 99999.0, 1.0);
    height_spin.set_value(orig_h);
    grid.attach(&height_spin, 1, 1, 1, 1);

    // Lock aspect ratio checkbox
    let lock_aspect = gtk4::CheckButton::with_label("Lock aspect ratio");
    lock_aspect.set_active(true);
    grid.attach(&lock_aspect, 0, 2, 2, 1);

    // Percentage input
    grid.attach(&Label::new(Some("Percentage:")), 0, 3, 1, 1);
    let pct_spin = SpinButton::with_range(1.0, 10000.0, 1.0);
    pct_spin.set_value(100.0);
    grid.attach(&pct_spin, 1, 3, 1, 1);

    content.append(&grid);

    // Link width/height when aspect locked
    let height_ref = height_spin.clone();
    let lock_ref = lock_aspect.clone();
    let aspect = orig_w / orig_h;
    width_spin.connect_value_changed(move |spin| {
        if lock_ref.is_active() {
            let new_h = (spin.value() / aspect).round();
            height_ref.set_value(new_h);
        }
    });

    let width_ref = width_spin.clone();
    let lock_ref2 = lock_aspect.clone();
    height_spin.connect_value_changed(move |spin| {
        if lock_ref2.is_active() {
            let new_w = (spin.value() * aspect).round();
            width_ref.set_value(new_w);
        }
    });

    // Link percentage to dimensions
    let w_pct = width_spin.clone();
    let h_pct = height_spin.clone();
    pct_spin.connect_value_changed(move |spin| {
        let pct = spin.value() / 100.0;
        w_pct.set_value((orig_w * pct).round());
        h_pct.set_value((orig_h * pct).round());
    });

    // Buttons
    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk4::Align::End);
    btn_box.set_margin_top(12);

    let cancel_btn = Button::with_label("Cancel");
    let dialog_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_cancel.close();
    });
    btn_box.append(&cancel_btn);

    let apply_btn = Button::with_label("Apply");
    apply_btn.add_css_class("suggested-action");
    let state = state.clone();
    let dialog_apply = dialog.clone();
    apply_btn.connect_clicked(move |_| {
        let new_w = width_spin.value() as u32;
        let new_h = height_spin.value() as u32;
        if new_w > 0 && new_h > 0 {
            apply_resize(&state, new_w, new_h);
        }
        dialog_apply.close();
    });
    btn_box.append(&apply_btn);

    content.append(&btn_box);
    dialog.set_child(Some(&content));
    dialog.present();
}

fn apply_resize(state: &Rc<RefCell<AppState>>, new_w: u32, new_h: u32) {
    let mut s = state.borrow_mut();
    let Some(pixbuf) = &s.current_pixbuf else {
        return;
    };

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

    let result = editing::resize(&img, new_w, new_h);
    let rgba = result.to_rgba8();
    let (rw, rh) = (rgba.width(), rgba.height());
    let raw = rgba.into_raw();

    let format = s
        .current_image
        .as_ref()
        .map(|img| img.format)
        .unwrap_or(crate::formats::ImageFormat::Png);
    s.current_image = Some(crate::formats::DecodedImage {
        width: rw,
        height: rh,
        pixels: raw.clone(),
        format,
    });

    let new_pixbuf = gtk4::gdk_pixbuf::Pixbuf::from_bytes(
        &glib::Bytes::from(&raw),
        gtk4::gdk_pixbuf::Colorspace::Rgb,
        true,
        8,
        rw as i32,
        rh as i32,
        (rw * 4) as i32,
    );
    s.current_pixbuf = Some(new_pixbuf);
    s.has_unsaved_edits = true;

    let cb = s.on_image_changed.clone();
    drop(s);
    if let Some(cb) = cb {
        cb();
    }
}

/// Show the crop dialog with aspect ratio presets.
/// The crop is applied as a simple input dialog for now (top, left, width, height).
pub fn show_crop_dialog(window: &ApplicationWindow, state: &Rc<RefCell<AppState>>) {
    let s = state.borrow();
    let Some(pixbuf) = &s.current_pixbuf else {
        return;
    };
    let orig_w = pixbuf.width() as f64;
    let orig_h = pixbuf.height() as f64;
    drop(s);

    let dialog = gtk4::Window::builder()
        .title("Crop Image")
        .transient_for(window)
        .modal(true)
        .default_width(320)
        .default_height(280)
        .build();

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_margin_start(16);
    content.set_margin_end(16);

    // Aspect ratio preset
    let preset_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    preset_box.append(&Label::new(Some("Preset:")));
    let presets = StringList::new(&["Free", "Original", "1:1", "4:3", "16:9", "3:2"]);
    let preset = DropDown::new(Some(presets), None::<gtk4::Expression>);
    preset.set_selected(0);
    preset_box.append(&preset);
    content.append(&preset_box);

    // Crop region inputs
    let grid = gtk4::Grid::new();
    grid.set_row_spacing(8);
    grid.set_column_spacing(8);

    grid.attach(&Label::new(Some("X:")), 0, 0, 1, 1);
    let x_spin = SpinButton::with_range(0.0, orig_w - 1.0, 1.0);
    x_spin.set_value(0.0);
    grid.attach(&x_spin, 1, 0, 1, 1);

    grid.attach(&Label::new(Some("Y:")), 0, 1, 1, 1);
    let y_spin = SpinButton::with_range(0.0, orig_h - 1.0, 1.0);
    y_spin.set_value(0.0);
    grid.attach(&y_spin, 1, 1, 1, 1);

    grid.attach(&Label::new(Some("Width:")), 0, 2, 1, 1);
    let w_spin = SpinButton::with_range(1.0, orig_w, 1.0);
    w_spin.set_value(orig_w);
    grid.attach(&w_spin, 1, 2, 1, 1);

    grid.attach(&Label::new(Some("Height:")), 0, 3, 1, 1);
    let h_spin = SpinButton::with_range(1.0, orig_h, 1.0);
    h_spin.set_value(orig_h);
    grid.attach(&h_spin, 1, 3, 1, 1);

    content.append(&grid);

    // Preset changes adjust width/height
    let w_preset = w_spin.clone();
    let h_preset = h_spin.clone();
    preset.connect_selected_notify(move |dd| {
        let ratio: Option<f64> = match dd.selected() {
            1 => Some(orig_w / orig_h), // Original
            2 => Some(1.0),             // 1:1
            3 => Some(4.0 / 3.0),       // 4:3
            4 => Some(16.0 / 9.0),      // 16:9
            5 => Some(3.0 / 2.0),       // 3:2
            _ => None,                  // Free
        };
        if let Some(r) = ratio {
            let w = w_preset.value();
            let h = (w / r).round().min(orig_h);
            h_preset.set_value(h);
        }
    });

    // Buttons
    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk4::Align::End);
    btn_box.set_margin_top(12);

    let cancel_btn = Button::with_label("Cancel");
    let dialog_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_cancel.close();
    });
    btn_box.append(&cancel_btn);

    let apply_btn = Button::with_label("Crop");
    apply_btn.add_css_class("suggested-action");
    let state = state.clone();
    let dialog_apply = dialog.clone();
    apply_btn.connect_clicked(move |_| {
        let cx = x_spin.value() as u32;
        let cy = y_spin.value() as u32;
        let cw = w_spin.value() as u32;
        let ch = h_spin.value() as u32;
        apply_crop(&state, cx, cy, cw, ch);
        dialog_apply.close();
    });
    btn_box.append(&apply_btn);

    content.append(&btn_box);
    dialog.set_child(Some(&content));
    dialog.present();
}

fn apply_crop(state: &Rc<RefCell<AppState>>, x: u32, y: u32, w: u32, h: u32) {
    let mut s = state.borrow_mut();
    let Some(pixbuf) = &s.current_pixbuf else {
        return;
    };

    let Some(bytes) = pixbuf.pixel_bytes() else {
        return;
    };
    let img_w = pixbuf.width() as u32;
    let img_h = pixbuf.height() as u32;

    // Bounds check
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

    let format = s
        .current_image
        .as_ref()
        .map(|img| img.format)
        .unwrap_or(crate::formats::ImageFormat::Png);
    s.current_image = Some(crate::formats::DecodedImage {
        width: cw,
        height: ch,
        pixels: raw.clone(),
        format,
    });

    let new_pixbuf = gtk4::gdk_pixbuf::Pixbuf::from_bytes(
        &glib::Bytes::from(&raw),
        gtk4::gdk_pixbuf::Colorspace::Rgb,
        true,
        8,
        cw as i32,
        ch as i32,
        (cw * 4) as i32,
    );
    s.current_pixbuf = Some(new_pixbuf);
    s.has_unsaved_edits = true;

    let cb = s.on_image_changed.clone();
    drop(s);
    if let Some(cb) = cb {
        cb();
    }
}
