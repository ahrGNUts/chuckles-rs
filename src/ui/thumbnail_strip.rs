use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::{Picture, ScrolledWindow};

use crate::ui::state::{self, AppState};

const THUMB_SIZE: i32 = 64;

/// Build the thumbnail strip widget.
pub fn build_thumbnail_strip(state: &Rc<RefCell<AppState>>) -> ScrolledWindow {
    let scrolled = ScrolledWindow::new();
    scrolled.set_min_content_height(THUMB_SIZE + 16);
    scrolled.set_max_content_height(THUMB_SIZE + 16);
    scrolled.set_hexpand(true);
    scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Never);

    let content = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    content.set_margin_start(4);
    content.set_margin_end(4);
    content.set_margin_top(4);
    content.set_margin_bottom(4);

    scrolled.set_child(Some(&content));
    scrolled
}

/// Rebuild the thumbnail strip content for the current directory.
pub fn update_thumbnails(strip: &ScrolledWindow, state: &Rc<RefCell<AppState>>) {
    let content = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    content.set_margin_start(4);
    content.set_margin_end(4);
    content.set_margin_top(4);
    content.set_margin_bottom(4);

    let s = state.borrow();
    let current_idx = s.image_list.current_index();

    for i in 0..s.image_list.len() {
        let Some(path) = s.image_list.path_at(i) else {
            continue;
        };

        let frame = gtk4::Frame::new(None);

        // Try to load a thumbnail
        if let Ok(pixbuf) = Pixbuf::from_file_at_scale(path, THUMB_SIZE, THUMB_SIZE, true) {
            let picture = Picture::for_pixbuf(&pixbuf);
            picture.set_size_request(THUMB_SIZE, THUMB_SIZE);
            frame.set_child(Some(&picture));
        } else {
            let label = gtk4::Label::new(Some("?"));
            label.set_size_request(THUMB_SIZE, THUMB_SIZE);
            frame.set_child(Some(&label));
        }

        // Highlight current image
        if current_idx == Some(i) {
            frame.add_css_class("thumbnail-current");
        }

        // Click to navigate
        let gesture = gtk4::GestureClick::new();
        let state_click = state.clone();
        gesture.connect_released(move |_, _, _, _| {
            state::navigate_to_index(&state_click, i);
        });
        frame.add_controller(gesture);

        content.append(&frame);
    }

    drop(s);
    strip.set_child(Some(&content));
}
