use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Label, ScrolledWindow};

use crate::metadata;
use crate::ui::state::AppState;

pub fn build_metadata_panel(state: &Rc<RefCell<AppState>>) -> ScrolledWindow {
    let scrolled = ScrolledWindow::new();
    scrolled.set_width_request(250);
    scrolled.set_vexpand(true);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    scrolled.set_child(Some(&content));
    scrolled
}

/// Rebuild the metadata panel content for the current image.
pub fn update_metadata_content(panel: &ScrolledWindow, state: &AppState) {
    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let Some(decoded) = &state.current_image else {
        let label = Label::new(Some("No image loaded"));
        label.add_css_class("dim-label");
        content.append(&label);
        panel.set_child(Some(&content));
        return;
    };

    // File info section
    add_section_header(&content, "File Info");
    if let Some(path) = state.image_list.current_path() {
        let info = metadata::FileInfo::from_path(
            path,
            decoded.width,
            decoded.height,
            decoded.color_depth,
            decoded.format,
        );
        if let Some(info) = info {
            add_field(&content, "Name", &info.filename);
            add_field(&content, "Path", &info.path);
            add_field(&content, "Size", &info.formatted_size());
            add_field(&content, "Dimensions", &info.formatted_dimensions());
            add_field(
                &content,
                "Color Depth",
                &format!("{} bit", info.color_depth),
            );
            add_field(&content, "Format", &format!("{:?}", info.format));
        }
    }

    // EXIF section
    if let Some(path) = state.image_list.current_path() {
        if let Some(exif) = metadata::exif_reader::read_exif(path) {
            if exif.has_camera_data() {
                add_section_header(&content, "Camera");
                if let Some(v) = &exif.camera_make {
                    add_field(&content, "Make", v);
                }
                if let Some(v) = &exif.camera_model {
                    add_field(&content, "Model", v);
                }
                if let Some(v) = &exif.lens {
                    add_field(&content, "Lens", v);
                }
                if let Some(v) = &exif.aperture {
                    add_field(&content, "Aperture", v);
                }
                if let Some(v) = &exif.shutter_speed {
                    add_field(&content, "Shutter", v);
                }
                if let Some(v) = &exif.iso {
                    add_field(&content, "ISO", v);
                }
                if let Some(v) = &exif.focal_length {
                    add_field(&content, "Focal Length", v);
                }
                if let Some(v) = &exif.date_taken {
                    add_field(&content, "Date Taken", v);
                }
                if let Some(v) = &exif.gps_latitude {
                    add_field(&content, "GPS Lat", v);
                }
                if let Some(v) = &exif.gps_longitude {
                    add_field(&content, "GPS Long", v);
                }
            }

            // Raw tags section
            if !exif.raw_tags.is_empty() {
                add_section_header(&content, "Raw EXIF");
                for (tag, value) in &exif.raw_tags {
                    add_field(&content, tag, value);
                }
            }
        } else {
            add_section_header(&content, "EXIF");
            let label = Label::new(Some("No EXIF data"));
            label.add_css_class("dim-label");
            content.append(&label);
        }
    }

    panel.set_child(Some(&content));
}

fn add_section_header(container: &gtk4::Box, title: &str) {
    let label = Label::new(Some(title));
    label.set_halign(gtk4::Align::Start);
    label.add_css_class("heading");
    label.set_margin_top(12);
    container.append(&label);
}

fn add_field(container: &gtk4::Box, name: &str, value: &str) {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let name_label = Label::new(Some(name));
    name_label.add_css_class("dim-label");
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_width_request(100);

    let value_label = Label::new(Some(value));
    value_label.set_halign(gtk4::Align::Start);
    value_label.set_wrap(true);
    value_label.set_selectable(true);
    value_label.set_hexpand(true);

    row.append(&name_label);
    row.append(&value_label);
    container.append(&row);
}
