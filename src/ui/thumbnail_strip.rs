use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::{Picture, ScrolledWindow};

use crate::ui::state::{self, AppState};

const THUMB_SIZE: i32 = 64;
/// Number of thumbnails to eagerly decode on each side of the current image.
const VISIBLE_WINDOW: usize = 20;

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
/// Only decodes thumbnails within VISIBLE_WINDOW of the current image;
/// all others get placeholder frames (lazy loading).
pub fn update_thumbnails(strip: &ScrolledWindow, state: &Rc<RefCell<AppState>>) {
    let content = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    content.set_margin_start(4);
    content.set_margin_end(4);
    content.set_margin_top(4);
    content.set_margin_bottom(4);

    let s = state.borrow();
    let current_idx = s.image_list.current_index().unwrap_or(0);
    let len = s.image_list.len();

    let visible_start = current_idx.saturating_sub(VISIBLE_WINDOW);
    let visible_end = (current_idx + VISIBLE_WINDOW + 1).min(len);

    for i in 0..len {
        let Some(path) = s.image_list.path_at(i) else {
            continue;
        };

        let frame = gtk4::Frame::new(None);
        frame.set_size_request(THUMB_SIZE + 4, THUMB_SIZE + 4);

        if i >= visible_start && i < visible_end {
            // Within visible window: decode the thumbnail
            if let Ok(pixbuf) = Pixbuf::from_file_at_scale(path, THUMB_SIZE, THUMB_SIZE, true) {
                let picture = Picture::for_pixbuf(&pixbuf);
                picture.set_size_request(THUMB_SIZE, THUMB_SIZE);
                frame.set_child(Some(&picture));
            } else {
                let label = gtk4::Label::new(Some("?"));
                label.set_size_request(THUMB_SIZE, THUMB_SIZE);
                frame.set_child(Some(&label));
            }
        } else {
            // Outside visible window: placeholder
            let placeholder = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            placeholder.set_size_request(THUMB_SIZE, THUMB_SIZE);
            placeholder.add_css_class("dim-label");
            frame.set_child(Some(&placeholder));
        }

        // Highlight current image
        if s.image_list.current_index() == Some(i) {
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

    // Scroll to show the current thumbnail
    scroll_to_current(strip, current_idx, len);
}

fn scroll_to_current(strip: &ScrolledWindow, current: usize, total: usize) {
    if total == 0 {
        return;
    }
    let adj = strip.hadjustment();
    let thumb_width = (THUMB_SIZE + 8) as f64; // frame + gap
    let target = current as f64 * thumb_width;
    let page = adj.page_size();
    // Center the current thumbnail in the strip
    adj.set_value(target - page / 2.0 + thumb_width / 2.0);
}
