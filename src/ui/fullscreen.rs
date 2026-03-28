use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Overlay, Revealer, RevealerTransitionType};

use crate::ui::state::AppState;

/// Setup fullscreen fly-out panels on the overlay.
/// Panels appear on mouse hover at screen edges:
/// - Bottom: thumbnail strip + navigation
/// - Right: metadata panel
/// - Left: edit tools
pub fn setup_fullscreen_panels(overlay: &Overlay, state: &Rc<RefCell<AppState>>) {
    // Bottom edge detection zone (thumbnails + nav)
    let bottom_revealer = Revealer::new();
    bottom_revealer.set_transition_type(RevealerTransitionType::SlideUp);
    bottom_revealer.set_reveal_child(false);
    bottom_revealer.set_valign(gtk4::Align::End);
    bottom_revealer.set_hexpand(true);

    let bottom_label = gtk4::Label::new(Some("Thumbnails + Navigation"));
    bottom_label.set_height_request(80);
    bottom_label.add_css_class("osd");
    bottom_revealer.set_child(Some(&bottom_label));

    // Right edge detection zone (metadata)
    let right_revealer = Revealer::new();
    right_revealer.set_transition_type(RevealerTransitionType::SlideLeft);
    right_revealer.set_reveal_child(false);
    right_revealer.set_halign(gtk4::Align::End);
    right_revealer.set_vexpand(true);

    let right_label = gtk4::Label::new(Some("Metadata"));
    right_label.set_width_request(200);
    right_label.add_css_class("osd");
    right_revealer.set_child(Some(&right_label));

    // Left edge detection zone (edit tools)
    let left_revealer = Revealer::new();
    left_revealer.set_transition_type(RevealerTransitionType::SlideRight);
    left_revealer.set_reveal_child(false);
    left_revealer.set_halign(gtk4::Align::Start);
    left_revealer.set_vexpand(true);

    let left_label = gtk4::Label::new(Some("Edit Tools"));
    left_label.set_width_request(160);
    left_label.add_css_class("osd");
    left_revealer.set_child(Some(&left_label));

    overlay.add_overlay(&bottom_revealer);
    overlay.add_overlay(&right_revealer);
    overlay.add_overlay(&left_revealer);

    // Edge detection via motion controller
    let motion = gtk4::EventControllerMotion::new();
    let bottom_rev = bottom_revealer.clone();
    let right_rev = right_revealer.clone();
    let left_rev = left_revealer.clone();
    let state_motion = state.clone();

    motion.connect_motion(move |ctrl, x, y| {
        let is_fullscreen = state_motion.borrow().is_fullscreen;
        if !is_fullscreen {
            // Hide all fullscreen panels when not in fullscreen
            bottom_rev.set_reveal_child(false);
            right_rev.set_reveal_child(false);
            left_rev.set_reveal_child(false);
            return;
        }

        let Some(widget) = ctrl.widget() else {
            return;
        };
        let width = widget.width() as f64;
        let height = widget.height() as f64;
        let edge_zone = 20.0;

        bottom_rev.set_reveal_child(y > height - edge_zone);
        right_rev.set_reveal_child(x > width - edge_zone);
        left_rev.set_reveal_child(x < edge_zone);
    });

    let bottom_leave = bottom_revealer.clone();
    let right_leave = right_revealer.clone();
    let left_leave = left_revealer.clone();
    motion.connect_leave(move |_| {
        bottom_leave.set_reveal_child(false);
        right_leave.set_reveal_child(false);
        left_leave.set_reveal_child(false);
    });

    overlay.add_controller(motion);
}
