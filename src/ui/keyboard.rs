use std::cell::RefCell;
use std::rc::Rc;

use gtk4::ApplicationWindow;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;

use crate::ui::state::{self, AppState, NavigateAction};
use crate::ui::toolbar::{self, EditAction};

pub fn setup_key_handler(window: &ApplicationWindow, state: &Rc<RefCell<AppState>>) {
    let controller = gtk4::EventControllerKey::new();
    let state = state.clone();
    let window_ref = window.clone();

    controller.connect_key_pressed(move |_, keyval, _keycode, modifier| {
        let ctrl = modifier.contains(gdk::ModifierType::CONTROL_MASK);
        let shift = modifier.contains(gdk::ModifierType::SHIFT_MASK);

        match keyval {
            // Navigation
            gdk::Key::Left => {
                state::navigate(&state, NavigateAction::Prev);
                glib::Propagation::Stop
            }
            gdk::Key::Right => {
                state::navigate(&state, NavigateAction::Next);
                glib::Propagation::Stop
            }
            gdk::Key::Home => {
                state::navigate(&state, NavigateAction::First);
                glib::Propagation::Stop
            }
            gdk::Key::End => {
                state::navigate(&state, NavigateAction::Last);
                glib::Propagation::Stop
            }

            // Zoom
            gdk::Key::plus | gdk::Key::equal => {
                state.borrow_mut().zoom_in();
                glib::Propagation::Stop
            }
            gdk::Key::minus => {
                state.borrow_mut().zoom_out();
                glib::Propagation::Stop
            }
            gdk::Key::_1 => {
                state.borrow_mut().zoom_actual();
                glib::Propagation::Stop
            }
            gdk::Key::f if !ctrl => {
                state.borrow_mut().zoom_fit();
                glib::Propagation::Stop
            }

            // Enter: confirm crop (if in crop mode) or toggle full-screen
            gdk::Key::Return => {
                let in_crop = state.borrow().crop_state.is_some();
                if in_crop {
                    crate::ui::crop_overlay::apply_crop(&state);
                } else {
                    let mut s = state.borrow_mut();
                    s.is_fullscreen = !s.is_fullscreen;
                    if let Some(cb) = &s.on_panels_changed {
                        cb();
                    }
                }
                glib::Propagation::Stop
            }
            gdk::Key::F11 => {
                let mut s = state.borrow_mut();
                s.is_fullscreen = !s.is_fullscreen;
                if let Some(cb) = &s.on_panels_changed {
                    cb();
                }
                glib::Propagation::Stop
            }
            // Escape: cancel crop (if in crop mode) or exit full-screen
            gdk::Key::Escape => {
                let in_crop = state.borrow().crop_state.is_some();
                if in_crop {
                    let mut s = state.borrow_mut();
                    s.crop_state = None;
                    if let Some(cb) = &s.on_zoom_changed {
                        cb();
                    }
                } else {
                    let mut s = state.borrow_mut();
                    if s.is_fullscreen {
                        s.is_fullscreen = false;
                        if let Some(cb) = &s.on_panels_changed {
                            cb();
                        }
                    }
                }
                glib::Propagation::Stop
            }

            // Editing shortcuts
            gdk::Key::r if !ctrl => {
                toolbar::apply_edit_action(&state, EditAction::RotateRight);
                glib::Propagation::Stop
            }
            gdk::Key::l if !ctrl => {
                toolbar::apply_edit_action(&state, EditAction::RotateLeft);
                glib::Propagation::Stop
            }
            gdk::Key::h if !ctrl => {
                toolbar::apply_edit_action(&state, EditAction::FlipHorizontal);
                glib::Propagation::Stop
            }
            gdk::Key::v if !ctrl => {
                toolbar::apply_edit_action(&state, EditAction::FlipVertical);
                glib::Propagation::Stop
            }
            gdk::Key::x if !ctrl => {
                toolbar::apply_edit_action(&state, EditAction::Crop);
                glib::Propagation::Stop
            }

            // Panel toggles
            gdk::Key::i if !ctrl => {
                let mut s = state.borrow_mut();
                s.sidebar_visible = !s.sidebar_visible;
                if let Some(cb) = &s.on_panels_changed {
                    cb();
                }
                glib::Propagation::Stop
            }
            gdk::Key::t if !ctrl => {
                let mut s = state.borrow_mut();
                s.thumbnail_strip_visible = !s.thumbnail_strip_visible;
                if let Some(cb) = &s.on_panels_changed {
                    cb();
                }
                glib::Propagation::Stop
            }

            // Ctrl+Q: quit
            gdk::Key::q if ctrl => {
                window_ref.close();
                glib::Propagation::Stop
            }

            // Ctrl+Shift+S: Save As
            gdk::Key::S if ctrl && shift => {
                let s = state.borrow();
                if s.has_unsaved_edits {
                    drop(s);
                    show_save_as_dialog(&window_ref, &state);
                }
                glib::Propagation::Stop
            }

            _ => glib::Propagation::Proceed,
        }
    });

    window.add_controller(controller);
}

fn show_save_as_dialog(window: &ApplicationWindow, state: &Rc<RefCell<AppState>>) {
    let dialog = gtk4::FileDialog::builder()
        .title("Save As")
        .modal(true)
        .build();

    // Set initial filename from current image
    let initial_name = state
        .borrow()
        .image_list
        .current_path()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "image.png".to_string());
    dialog.set_initial_name(Some(&initial_name));

    let state = state.clone();
    dialog.save(
        Some(window),
        None::<&gtk4::gio::Cancellable>,
        move |result| {
            let Ok(file) = result else { return };
            let Some(path) = file.path() else { return };
            let s = state.borrow();
            let Some(pixbuf) = &s.current_pixbuf else {
                return;
            };
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png")
                .to_lowercase();
            let format = match ext.as_str() {
                "jpg" | "jpeg" => "jpeg",
                "png" => "png",
                "bmp" => "bmp",
                "tiff" | "tif" => "tiff",
                _ => "png",
            };
            let options: &[(&str, &str)] = if format == "jpeg" {
                &[("quality", "100")]
            } else {
                &[]
            };
            if let Err(e) = pixbuf.savev(&path, format, options) {
                eprintln!("Error saving: {e}");
            } else {
                drop(s);
                state.borrow_mut().has_unsaved_edits = false;
            }
        },
    );
}
