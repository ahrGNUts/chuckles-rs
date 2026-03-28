use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Button, HeaderBar, Label, MenuButton, Popover};

use crate::config::{SortDirection, SortMode};
use crate::ui::state::AppState;

pub struct HeaderWidgets {
    pub header_bar: HeaderBar,
    pub title_label: Label,
    pub index_label: Label,
}

pub fn build_header_bar(state: &Rc<RefCell<AppState>>) -> HeaderWidgets {
    let header_bar = HeaderBar::new();

    let title_label = Label::new(Some("chuckles"));
    title_label.add_css_class("title");

    let index_label = Label::new(None);
    index_label.add_css_class("subtitle");

    let title_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    title_box.append(&title_label);
    title_box.append(&index_label);
    header_bar.set_title_widget(Some(&title_box));

    // Sort menu
    let sort_button = build_sort_menu(state);
    header_bar.pack_start(&sort_button);

    // Zoom buttons
    let zoom_fit = Button::with_label("Fit");
    let state_fit = state.clone();
    zoom_fit.connect_clicked(move |_| {
        state_fit.borrow_mut().zoom_fit();
    });
    header_bar.pack_start(&zoom_fit);

    let zoom_actual = Button::with_label("1:1");
    let state_actual = state.clone();
    zoom_actual.connect_clicked(move |_| {
        state_actual.borrow_mut().zoom_actual();
    });
    header_bar.pack_start(&zoom_actual);

    // Edit toggle
    let edit_button = Button::with_label("Edit");
    let state_edit = state.clone();
    edit_button.connect_clicked(move |_| {
        let mut s = state_edit.borrow_mut();
        s.edit_panel_visible = !s.edit_panel_visible;
        if let Some(cb) = &s.on_panels_changed {
            cb();
        }
    });
    header_bar.pack_end(&edit_button);

    HeaderWidgets {
        header_bar,
        title_label,
        index_label,
    }
}

fn build_sort_menu(state: &Rc<RefCell<AppState>>) -> MenuButton {
    let sort_button = MenuButton::new();
    sort_button.set_label("Sort");

    let popover = Popover::new();
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    vbox.set_margin_top(8);
    vbox.set_margin_bottom(8);
    vbox.set_margin_start(8);
    vbox.set_margin_end(8);

    let sort_options = [
        ("Name", SortMode::Name),
        ("Date Modified", SortMode::DateModified),
        ("File Size", SortMode::FileSize),
        ("File Type", SortMode::FileType),
        ("Dimensions", SortMode::Dimensions),
    ];

    for (label, mode) in sort_options {
        let btn = Button::with_label(label);
        let state = state.clone();
        let popover_ref = popover.clone();
        btn.connect_clicked(move |_| {
            let mut s = state.borrow_mut();
            let dir = s.image_list.sort_direction();
            s.image_list.set_sort(mode, dir);
            if let Some(cb) = &s.on_list_changed {
                cb();
            }
            popover_ref.popdown();
        });
        vbox.append(&btn);
    }

    // Direction toggle
    let dir_btn = Button::with_label("Toggle Direction");
    let state_dir = state.clone();
    let popover_dir = popover.clone();
    dir_btn.connect_clicked(move |_| {
        let mut s = state_dir.borrow_mut();
        s.image_list.toggle_direction();
        if let Some(cb) = &s.on_list_changed {
            cb();
        }
        popover_dir.popdown();
    });
    vbox.append(&dir_btn);

    popover.set_child(Some(&vbox));
    sort_button.set_popover(Some(&popover));

    sort_button
}
