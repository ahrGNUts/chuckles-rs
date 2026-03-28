use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Overlay, Revealer, RevealerTransitionType, ScrolledWindow};

use crate::ui::{
    canvas, fullscreen, header, metadata_panel, state::AppState, thumbnail_strip, toolbar,
};

/// Build the main window layout with all panels.
pub fn build_main_layout(window: &ApplicationWindow, state: &Rc<RefCell<AppState>>) -> gtk4::Box {
    let header_widgets = header::build_header_bar(state);
    window.set_titlebar(Some(&header_widgets.header_bar));

    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // Edit toolbar (toggleable)
    let edit_toolbar = toolbar::build_edit_toolbar(state);
    let edit_revealer = Revealer::new();
    edit_revealer.set_child(Some(&edit_toolbar));
    edit_revealer.set_transition_type(RevealerTransitionType::SlideDown);
    edit_revealer.set_reveal_child(false);
    root.append(&edit_revealer);

    // Middle section: canvas + sidebar
    let middle = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    middle.set_vexpand(true);

    // Image canvas with overlay for fullscreen panels and navigation arrows
    let image_canvas = canvas::build_canvas(state);
    let overlay = Overlay::new();
    overlay.set_child(Some(&image_canvas));
    overlay.set_hexpand(true);
    overlay.set_vexpand(true);

    // Navigation arrow overlays
    let left_arrow = build_nav_arrow("go-previous-symbolic", state, true);
    left_arrow.set_halign(gtk4::Align::Start);
    left_arrow.set_valign(gtk4::Align::Center);
    overlay.add_overlay(&left_arrow);

    let right_arrow = build_nav_arrow("go-next-symbolic", state, false);
    right_arrow.set_halign(gtk4::Align::End);
    right_arrow.set_valign(gtk4::Align::Center);
    overlay.add_overlay(&right_arrow);

    middle.append(&overlay);

    // Metadata sidebar (toggleable)
    let sidebar = metadata_panel::build_metadata_panel(state);
    let sidebar_revealer = Revealer::new();
    sidebar_revealer.set_child(Some(&sidebar));
    sidebar_revealer.set_transition_type(RevealerTransitionType::SlideLeft);
    sidebar_revealer.set_reveal_child(false);
    middle.append(&sidebar_revealer);

    root.append(&middle);

    // Thumbnail strip (toggleable)
    let thumb_strip = thumbnail_strip::build_thumbnail_strip(state);
    let thumb_revealer = Revealer::new();
    thumb_revealer.set_child(Some(&thumb_strip));
    thumb_revealer.set_transition_type(RevealerTransitionType::SlideUp);
    thumb_revealer.set_reveal_child(false);
    root.append(&thumb_revealer);

    // Wire up state callbacks to update UI
    let title_label = header_widgets.title_label.clone();
    let index_label = header_widgets.index_label.clone();
    let canvas_ref = image_canvas.clone();
    let sidebar_ref = sidebar.clone();
    let state_img = state.clone();
    state.borrow_mut().on_image_changed = Some(Rc::new(move || {
        let s = state_img.borrow();
        title_label.set_label(&s.header_title());
        let idx_text = if let (Some(idx), len) = (s.image_list.current_index(), s.image_list.len())
        {
            if len > 0 {
                format!("{} of {len}", idx + 1)
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        index_label.set_label(&idx_text);
        drop(s);
        metadata_panel::update_metadata_content(&sidebar_ref, &state_img.borrow());
        canvas_ref.queue_draw();
    }));

    let canvas_zoom = image_canvas.clone();
    state.borrow_mut().on_zoom_changed = Some(Rc::new(move || {
        canvas_zoom.queue_draw();
    }));

    let title_list = header_widgets.title_label.clone();
    let index_list = header_widgets.index_label.clone();
    let thumb_strip_ref = thumb_strip.clone();
    let state_list = state.clone();
    state.borrow_mut().on_list_changed = Some(Rc::new(move || {
        let s = state_list.borrow();
        title_list.set_label(&s.header_title());
        let idx_text = if let (Some(idx), len) = (s.image_list.current_index(), s.image_list.len())
        {
            if len > 0 {
                format!("{} of {len}", idx + 1)
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        index_list.set_label(&idx_text);
        drop(s);
        thumbnail_strip::update_thumbnails(&thumb_strip_ref, &state_list);
    }));

    let sidebar_rev = sidebar_revealer.clone();
    let thumb_rev = thumb_revealer.clone();
    let edit_rev = edit_revealer.clone();
    let header_bar = header_widgets.header_bar.clone();
    let window_ref = window.clone();
    let state_panels = state.clone();
    state.borrow_mut().on_panels_changed = Some(Rc::new(move || {
        let s = state_panels.borrow();
        sidebar_rev.set_reveal_child(s.sidebar_visible);
        thumb_rev.set_reveal_child(s.thumbnail_strip_visible);
        edit_rev.set_reveal_child(s.edit_panel_visible);

        if s.is_fullscreen {
            window_ref.fullscreen();
            header_bar.set_visible(false);
        } else {
            window_ref.unfullscreen();
            header_bar.set_visible(true);
        }
    }));

    // Set up fullscreen edge detection
    fullscreen::setup_fullscreen_panels(&overlay, state);

    root
}

fn build_nav_arrow(icon_name: &str, state: &Rc<RefCell<AppState>>, is_prev: bool) -> gtk4::Button {
    let btn = gtk4::Button::from_icon_name(icon_name);
    btn.add_css_class("circular");
    btn.add_css_class("osd");
    btn.set_margin_start(8);
    btn.set_margin_end(8);
    btn.set_opacity(0.6);

    let state = state.clone();
    btn.connect_clicked(move |_| {
        if is_prev {
            super::state::navigate(&state, super::state::NavigateAction::Prev);
        } else {
            super::state::navigate(&state, super::state::NavigateAction::Next);
        }
    });

    btn
}
