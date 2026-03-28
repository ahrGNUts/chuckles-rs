use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Overlay, Revealer, RevealerTransitionType};

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

    // Navigation arrow overlays (hidden by default, shown on hover)
    let left_arrow = build_nav_arrow("go-previous-symbolic", state, true);
    left_arrow.set_halign(gtk4::Align::Start);
    left_arrow.set_valign(gtk4::Align::Center);
    left_arrow.set_visible(false);
    overlay.add_overlay(&left_arrow);

    let right_arrow = build_nav_arrow("go-next-symbolic", state, false);
    right_arrow.set_halign(gtk4::Align::End);
    right_arrow.set_valign(gtk4::Align::Center);
    right_arrow.set_visible(false);
    overlay.add_overlay(&right_arrow);

    // Show/hide arrows on mouse enter/leave + track cursor position for zoom anchoring
    let hover_ctrl = gtk4::EventControllerMotion::new();
    let left_show = left_arrow.clone();
    let right_show = right_arrow.clone();
    hover_ctrl.connect_enter(move |_, _, _| {
        left_show.set_visible(true);
        right_show.set_visible(true);
    });
    let left_hide = left_arrow.clone();
    let right_hide = right_arrow.clone();
    hover_ctrl.connect_leave(move |_| {
        left_hide.set_visible(false);
        right_hide.set_visible(false);
    });
    let state_motion = state.clone();
    hover_ctrl.connect_motion(move |_, x, y| {
        state_motion.borrow_mut().last_mouse_pos = Some((x, y));
    });
    overlay.add_controller(hover_ctrl);

    middle.append(&overlay);

    // Metadata sidebar (toggleable)
    let sidebar = metadata_panel::build_metadata_panel(state);
    let sidebar_revealer = Revealer::new();
    sidebar_revealer.set_child(Some(&sidebar));
    sidebar_revealer.set_transition_type(RevealerTransitionType::SlideLeft);
    sidebar_revealer.set_reveal_child(state.borrow().sidebar_visible);
    middle.append(&sidebar_revealer);

    root.append(&middle);

    // Thumbnail strip (toggleable)
    let thumb_strip = thumbnail_strip::build_thumbnail_strip(state);
    let thumb_revealer = Revealer::new();
    thumb_revealer.set_child(Some(&thumb_strip));
    thumb_revealer.set_transition_type(RevealerTransitionType::SlideUp);
    thumb_revealer.set_reveal_child(state.borrow().thumbnail_strip_visible);
    root.append(&thumb_revealer);

    // Set up fullscreen edge detection with real panels
    let fs_panels = fullscreen::setup_fullscreen_panels(&overlay, state);

    // Wire up state callbacks to update UI
    let widgets = LayoutWidgets {
        header: header_widgets,
        canvas: image_canvas,
        sidebar,
        sidebar_revealer: &sidebar_revealer,
        thumb_strip,
        thumb_revealer: &thumb_revealer,
        edit_revealer: &edit_revealer,
        window: window.clone(),
        fs_panels,
    };
    wire_callbacks(state, &widgets);

    root
}

struct LayoutWidgets<'a> {
    header: header::HeaderWidgets,
    canvas: gtk4::DrawingArea,
    sidebar: gtk4::ScrolledWindow,
    sidebar_revealer: &'a Revealer,
    thumb_strip: gtk4::ScrolledWindow,
    thumb_revealer: &'a Revealer,
    edit_revealer: &'a Revealer,
    window: ApplicationWindow,
    fs_panels: fullscreen::FullscreenPanels,
}

fn wire_callbacks(state: &Rc<RefCell<AppState>>, w: &LayoutWidgets) {
    // on_image_changed: update title, metadata, canvas, fullscreen metadata
    let title_label = w.header.title_label.clone();
    let index_label = w.header.index_label.clone();
    let canvas_ref = w.canvas.clone();
    let sidebar_ref = w.sidebar.clone();
    let fs_metadata_ref = w.fs_panels.fs_metadata.clone();
    let state_img = state.clone();
    state.borrow_mut().on_image_changed = Some(Rc::new(move || {
        let s = state_img.borrow();
        title_label.set_label(&s.header_title());
        index_label.set_label(&s.header_subtitle());
        drop(s);
        metadata_panel::update_metadata_content(&sidebar_ref, &state_img.borrow());
        metadata_panel::update_metadata_content(&fs_metadata_ref, &state_img.borrow());
        canvas_ref.queue_draw();
    }));

    // on_zoom_changed: redraw canvas
    let canvas_zoom = w.canvas.clone();
    state.borrow_mut().on_zoom_changed = Some(Rc::new(move || {
        canvas_zoom.queue_draw();
    }));

    // on_list_changed: update title, thumbnails (both windowed and fullscreen)
    let title_list = w.header.title_label.clone();
    let index_list = w.header.index_label.clone();
    let thumb_strip_ref = w.thumb_strip.clone();
    let fs_thumb_ref = w.fs_panels.fs_thumb_strip.clone();
    let state_list = state.clone();
    state.borrow_mut().on_list_changed = Some(Rc::new(move || {
        let s = state_list.borrow();
        title_list.set_label(&s.header_title());
        index_list.set_label(&s.header_subtitle());
        drop(s);
        thumbnail_strip::update_thumbnails(&thumb_strip_ref, &state_list);
        thumbnail_strip::update_thumbnails(&fs_thumb_ref, &state_list);
    }));

    // on_panels_changed: toggle revealers, fullscreen
    let sidebar_rev = w.sidebar_revealer.clone();
    let thumb_rev = w.thumb_revealer.clone();
    let edit_rev = w.edit_revealer.clone();
    let header_bar = w.header.header_bar.clone();
    let window_ref = w.window.clone();
    let state_panels = state.clone();
    state.borrow_mut().on_panels_changed = Some(Rc::new(move || {
        let s = state_panels.borrow();
        sidebar_rev.set_reveal_child(s.sidebar_visible);
        thumb_rev.set_reveal_child(s.thumbnail_strip_visible);
        edit_rev.set_reveal_child(s.edit_panel_visible);

        if s.is_fullscreen {
            window_ref.fullscreen();
            header_bar.set_visible(false);
            // Hide windowed panels in fullscreen
            sidebar_rev.set_visible(false);
            thumb_rev.set_visible(false);
            edit_rev.set_visible(false);
        } else {
            window_ref.unfullscreen();
            header_bar.set_visible(true);
            sidebar_rev.set_visible(true);
            thumb_rev.set_visible(true);
            edit_rev.set_visible(true);
        }
    }));
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
