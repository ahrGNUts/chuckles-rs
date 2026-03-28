use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Button, Overlay, Revealer, RevealerTransitionType, ScrolledWindow};

use crate::ui::state::{self, AppState, NavigateAction};
use crate::ui::{metadata_panel, thumbnail_strip, toolbar};

/// Setup fullscreen fly-out panels on the overlay.
pub fn setup_fullscreen_panels(
    overlay: &Overlay,
    state: &Rc<RefCell<AppState>>,
) -> FullscreenPanels {
    // Bottom: thumbnail strip + navigation controls
    let bottom_revealer = Revealer::new();
    bottom_revealer.set_transition_type(RevealerTransitionType::SlideUp);
    bottom_revealer.set_reveal_child(false);
    bottom_revealer.set_valign(gtk4::Align::End);
    bottom_revealer.set_hexpand(true);

    let bottom_content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    bottom_content.add_css_class("osd");

    // Navigation bar within the bottom panel
    let nav_bar = build_fullscreen_nav_bar(state);
    bottom_content.append(&nav_bar);

    // Thumbnail strip within the bottom panel
    let fs_thumb_strip = thumbnail_strip::build_thumbnail_strip(state);
    bottom_content.append(&fs_thumb_strip);

    bottom_revealer.set_child(Some(&bottom_content));

    // Right: metadata panel
    let right_revealer = Revealer::new();
    right_revealer.set_transition_type(RevealerTransitionType::SlideLeft);
    right_revealer.set_reveal_child(false);
    right_revealer.set_halign(gtk4::Align::End);
    right_revealer.set_vexpand(true);

    let fs_metadata = metadata_panel::build_metadata_panel(state);
    fs_metadata.add_css_class("osd");
    right_revealer.set_child(Some(&fs_metadata));

    // Left: edit tools
    let left_revealer = Revealer::new();
    left_revealer.set_transition_type(RevealerTransitionType::SlideRight);
    left_revealer.set_reveal_child(false);
    left_revealer.set_halign(gtk4::Align::Start);
    left_revealer.set_vexpand(true);

    let fs_toolbar = build_fullscreen_edit_panel(state);
    fs_toolbar.add_css_class("osd");
    left_revealer.set_child(Some(&fs_toolbar));

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

    FullscreenPanels {
        fs_thumb_strip,
        fs_metadata,
    }
}

/// Handles to fullscreen panel widgets that need updating.
pub struct FullscreenPanels {
    pub fs_thumb_strip: ScrolledWindow,
    pub fs_metadata: ScrolledWindow,
}

fn build_fullscreen_nav_bar(state: &Rc<RefCell<AppState>>) -> gtk4::Box {
    let bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    bar.set_halign(gtk4::Align::Center);
    bar.set_margin_top(4);
    bar.set_margin_bottom(4);

    let prev_btn = Button::from_icon_name("go-previous-symbolic");
    let state_prev = state.clone();
    prev_btn.connect_clicked(move |_| {
        state::navigate(&state_prev, NavigateAction::Prev);
    });
    bar.append(&prev_btn);

    let next_btn = Button::from_icon_name("go-next-symbolic");
    let state_next = state.clone();
    next_btn.connect_clicked(move |_| {
        state::navigate(&state_next, NavigateAction::Next);
    });
    bar.append(&next_btn);

    let fit_btn = Button::with_label("Fit");
    let state_fit = state.clone();
    fit_btn.connect_clicked(move |_| {
        state_fit.borrow_mut().zoom_fit();
    });
    bar.append(&fit_btn);

    let actual_btn = Button::with_label("1:1");
    let state_actual = state.clone();
    actual_btn.connect_clicked(move |_| {
        state_actual.borrow_mut().zoom_actual();
    });
    bar.append(&actual_btn);

    bar
}

fn build_fullscreen_edit_panel(state: &Rc<RefCell<AppState>>) -> gtk4::Box {
    let panel = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    panel.set_margin_top(12);
    panel.set_margin_bottom(12);
    panel.set_margin_start(8);
    panel.set_margin_end(8);
    panel.set_width_request(140);

    let buttons = [
        ("Rotate Left", toolbar::EditAction::RotateLeft),
        ("Rotate Right", toolbar::EditAction::RotateRight),
        ("Flip H", toolbar::EditAction::FlipHorizontal),
        ("Flip V", toolbar::EditAction::FlipVertical),
        ("Crop", toolbar::EditAction::Crop),
        ("Resize", toolbar::EditAction::Resize),
    ];

    for (label, action) in buttons {
        let btn = Button::with_label(label);
        let state = state.clone();
        btn.connect_clicked(move |_| {
            toolbar::apply_edit_action(&state, action);
        });
        panel.append(&btn);
    }

    panel
}
