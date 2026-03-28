use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;

use crate::config::{ScrollWheelMode, SortDirection, SortMode, ZoomDefault};
use crate::formats::{self, DecodedImage};
use crate::viewer::ImageList;

/// Zoom level steps for incremental zoom.
const ZOOM_STEPS: &[f64] = &[
    0.05, 0.1, 0.15, 0.2, 0.25, 0.33, 0.5, 0.67, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0, 4.0, 5.0, 8.0,
    10.0,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZoomMode {
    Fit,
    Actual,
    Custom(f64),
}

/// Central application state shared across UI components.
pub struct AppState {
    pub image_list: ImageList,
    pub current_pixbuf: Option<Pixbuf>,
    pub current_image: Option<DecodedImage>,
    pub zoom: ZoomMode,
    pub default_zoom: ZoomDefault,
    pub scroll_wheel: ScrollWheelMode,
    pub background_color: String,
    pub has_unsaved_edits: bool,
    pub is_fullscreen: bool,
    pub sidebar_visible: bool,
    pub thumbnail_strip_visible: bool,
    pub edit_panel_visible: bool,
    pub crop_state: Option<super::crop_overlay::CropState>,
    pub pan_offset: (f64, f64),
    pub last_mouse_pos: Option<(f64, f64)>,
    /// The computed fit scale from the canvas, updated each draw.
    pub computed_fit_scale: f64,
    pub window: Option<gtk4::ApplicationWindow>,
    // Callbacks to update UI components
    pub on_image_changed: Option<Rc<dyn Fn()>>,
    pub on_zoom_changed: Option<Rc<dyn Fn()>>,
    pub on_list_changed: Option<Rc<dyn Fn()>>,
    pub on_panels_changed: Option<Rc<dyn Fn()>>,
}

impl AppState {
    pub fn new(
        sort_mode: SortMode,
        sort_direction: SortDirection,
        default_zoom: ZoomDefault,
        scroll_wheel: ScrollWheelMode,
        background_color: String,
        sidebar_visible: bool,
        thumbnail_strip_visible: bool,
    ) -> Self {
        Self {
            image_list: ImageList::new(sort_mode, sort_direction),
            current_pixbuf: None,
            current_image: None,
            zoom: match default_zoom {
                ZoomDefault::Fit => ZoomMode::Fit,
                ZoomDefault::Actual => ZoomMode::Actual,
            },
            default_zoom,
            scroll_wheel,
            background_color,
            has_unsaved_edits: false,
            is_fullscreen: false,
            sidebar_visible,
            thumbnail_strip_visible,
            edit_panel_visible: false,
            crop_state: None,
            pan_offset: (0.0, 0.0),
            last_mouse_pos: None,
            computed_fit_scale: 1.0,
            window: None,
            on_image_changed: None,
            on_zoom_changed: None,
            on_list_changed: None,
            on_panels_changed: None,
        }
    }

    pub fn zoom_in(&mut self) {
        let old = self.zoom_factor();
        let current = old;
        for &step in ZOOM_STEPS {
            if step > current + 0.001 {
                self.zoom = ZoomMode::Custom(step);
                // Scale pan offset to preserve viewport center
                self.scale_pan_offset(old, step);
                if let Some(cb) = &self.on_zoom_changed {
                    cb();
                }
                return;
            }
        }
    }

    pub fn zoom_out(&mut self) {
        let old = self.zoom_factor();
        let current = old;
        for &step in ZOOM_STEPS.iter().rev() {
            if step < current - 0.001 {
                self.zoom = ZoomMode::Custom(step);
                self.scale_pan_offset(old, step);
                if let Some(cb) = &self.on_zoom_changed {
                    cb();
                }
                return;
            }
        }
    }

    /// Scale the pan offset when zoom level changes to keep the viewport centered.
    fn scale_pan_offset(&mut self, old_zoom: f64, new_zoom: f64) {
        if old_zoom.abs() < 0.001 {
            return;
        }
        let ratio = new_zoom / old_zoom;
        self.pan_offset.0 *= ratio;
        self.pan_offset.1 *= ratio;
    }

    /// Zoom in/out anchored on a specific point (e.g., mouse cursor position).
    /// `anchor_x` and `anchor_y` are in canvas-relative coordinates.
    /// `canvas_w` and `canvas_h` are the canvas dimensions.
    pub fn zoom_at_point(
        &mut self,
        zoom_in: bool,
        anchor_x: f64,
        anchor_y: f64,
        canvas_w: f64,
        canvas_h: f64,
    ) {
        let old_zoom = self.zoom_factor();
        if zoom_in {
            let current = old_zoom;
            for &step in ZOOM_STEPS {
                if step > current + 0.001 {
                    self.zoom = ZoomMode::Custom(step);
                    self.adjust_pan_for_anchor(
                        old_zoom, step, anchor_x, anchor_y, canvas_w, canvas_h,
                    );
                    if let Some(cb) = &self.on_zoom_changed {
                        cb();
                    }
                    return;
                }
            }
        } else {
            let current = old_zoom;
            for &step in ZOOM_STEPS.iter().rev() {
                if step < current - 0.001 {
                    self.zoom = ZoomMode::Custom(step);
                    self.adjust_pan_for_anchor(
                        old_zoom, step, anchor_x, anchor_y, canvas_w, canvas_h,
                    );
                    if let Some(cb) = &self.on_zoom_changed {
                        cb();
                    }
                    return;
                }
            }
        }
    }

    /// Adjust pan offset so the point under the cursor stays in place after zoom.
    fn adjust_pan_for_anchor(
        &mut self,
        old_zoom: f64,
        new_zoom: f64,
        anchor_x: f64,
        anchor_y: f64,
        canvas_w: f64,
        canvas_h: f64,
    ) {
        // The anchor point in image space:
        // canvas_center + pan_offset maps to the center of the image
        // anchor relative to canvas center:
        let dx = anchor_x - canvas_w / 2.0;
        let dy = anchor_y - canvas_h / 2.0;

        // Scale the pan offset and adjust for the anchor point shift
        let ratio = new_zoom / old_zoom;
        self.pan_offset.0 = self.pan_offset.0 * ratio - dx * (ratio - 1.0);
        self.pan_offset.1 = self.pan_offset.1 * ratio - dy * (ratio - 1.0);
    }

    pub fn zoom_fit(&mut self) {
        self.zoom = ZoomMode::Fit;
        self.pan_offset = (0.0, 0.0);
        if let Some(cb) = &self.on_zoom_changed {
            cb();
        }
    }

    pub fn zoom_actual(&mut self) {
        self.zoom = ZoomMode::Actual;
        self.pan_offset = (0.0, 0.0);
        if let Some(cb) = &self.on_zoom_changed {
            cb();
        }
    }

    pub fn zoom_factor(&self) -> f64 {
        match self.zoom {
            ZoomMode::Custom(f) => f,
            ZoomMode::Actual => 1.0,
            ZoomMode::Fit => self.computed_fit_scale,
        }
    }

    pub fn header_title(&self) -> String {
        let filename = self
            .image_list
            .current_path()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "chuckles".to_string());

        if let (Some(idx), len) = (self.image_list.current_index(), self.image_list.len())
            && len > 0
        {
            return format!("{filename} ({} of {len})", idx + 1);
        }
        filename
    }

    pub fn header_subtitle(&self) -> String {
        if let (Some(idx), len) = (self.image_list.current_index(), self.image_list.len())
            && len > 0
        {
            return format!("{} of {len}", idx + 1);
        }
        String::new()
    }
}

/// Open a file: decode it immediately, then kick off background directory scan.
pub fn open_file(state: &Rc<RefCell<AppState>>, path: &Path) {
    // Decode and display the image immediately.
    match formats::decode_file(path) {
        Ok(decoded) => {
            let pixbuf = pixbuf_from_decoded(&decoded);
            {
                let mut s = state.borrow_mut();
                s.current_pixbuf = Some(pixbuf);
                s.current_image = Some(decoded);
                s.has_unsaved_edits = false;
                s.zoom = match s.default_zoom {
                    ZoomDefault::Fit => ZoomMode::Fit,
                    ZoomDefault::Actual => ZoomMode::Actual,
                };
                s.pan_offset = (0.0, 0.0);
            }
            notify_image_changed(state);
        }
        Err(e) => {
            show_decode_error(state, path, &e);
        }
    }

    // Background directory scan.
    let path = path.to_path_buf();
    let state = state.clone();
    let (sort_mode, sort_dir) = {
        let s = state.borrow();
        (s.image_list.sort_mode(), s.image_list.sort_direction())
    };
    if let Some(dir) = path.parent().map(|p| p.to_path_buf()) {
        glib::spawn_future_local(async move {
            let (entries_path, file_path) = (dir.clone(), path.clone());
            let result = gio::spawn_blocking(move || {
                let mut list = crate::viewer::ImageList::new(sort_mode, sort_dir);
                list.load_directory(&entries_path, &file_path);
                list
            })
            .await;

            if let Ok(list) = result {
                let mut s = state.borrow_mut();
                let sort_mode = s.image_list.sort_mode();
                let sort_dir = s.image_list.sort_direction();
                s.image_list = list;
                s.image_list.set_sort(sort_mode, sort_dir);
                if let Some(cb) = &s.on_list_changed {
                    cb();
                }
            }
        });
    }
}

/// Show the unsaved edits confirmation dialog. Calls `on_discard` if the user
/// chooses to discard, does nothing if they cancel.
/// Note: GTK4 AlertDialog is async so we cannot block for a return value.
/// Instead, callers pass a continuation closure.
pub fn confirm_discard_edits(state: &Rc<RefCell<AppState>>, on_discard: impl Fn() + 'static) {
    let has_edits = state.borrow().has_unsaved_edits;
    if !has_edits {
        on_discard();
        return;
    }

    let window = state.borrow().window.clone();
    let Some(win) = window else {
        // No window available, just discard
        on_discard();
        return;
    };

    let dialog = gtk4::AlertDialog::builder()
        .message("Unsaved Changes")
        .detail("You have unsaved edits. Discard them?")
        .buttons(["Cancel", "Discard"])
        .cancel_button(0)
        .default_button(1)
        .modal(true)
        .build();

    let state = state.clone();
    dialog.choose(Some(&win), None::<&gtk4::gio::Cancellable>, move |result| {
        if result == Ok(1) {
            // User chose "Discard"
            state.borrow_mut().has_unsaved_edits = false;
            on_discard();
        }
        // result == Ok(0) or Err => user cancelled, do nothing
    });
}

/// Navigate to the next/prev/first/last image.
/// If there are unsaved edits, shows a confirmation dialog first.
pub fn navigate(state: &Rc<RefCell<AppState>>, action: NavigateAction) {
    let state_for_dialog = state.clone();
    let state_for_nav = state.clone();
    confirm_discard_edits(&state_for_dialog, move || {
        do_navigate(&state_for_nav, action);
    });
}

fn do_navigate(state: &Rc<RefCell<AppState>>, action: NavigateAction) {
    let path = {
        let mut s = state.borrow_mut();
        let changed = match action {
            NavigateAction::Next => s.image_list.next(),
            NavigateAction::Prev => s.image_list.prev(),
            NavigateAction::First => s.image_list.first(),
            NavigateAction::Last => s.image_list.last(),
        };
        if !changed {
            return;
        }
        s.image_list.current_path().map(|p| p.to_path_buf())
    };

    if let Some(path) = path {
        load_image_at_path(state, &path);
    }
}

pub fn navigate_to_index(state: &Rc<RefCell<AppState>>, index: usize) {
    let state_for_dialog = state.clone();
    let state_for_nav = state.clone();
    let idx = index;
    confirm_discard_edits(&state_for_dialog, move || {
        let path = {
            let mut s = state_for_nav.borrow_mut();
            if !s.image_list.go_to(idx) {
                return;
            }
            s.image_list.current_path().map(|p| p.to_path_buf())
        };
        if let Some(path) = path {
            load_image_at_path(&state_for_nav, &path);
        }
    });
}

fn load_image_at_path(state: &Rc<RefCell<AppState>>, path: &Path) {
    match formats::decode_file(path) {
        Ok(decoded) => {
            let pixbuf = pixbuf_from_decoded(&decoded);
            {
                let mut s = state.borrow_mut();
                s.current_pixbuf = Some(pixbuf);
                s.current_image = Some(decoded);
                s.has_unsaved_edits = false;
                s.zoom = match s.default_zoom {
                    ZoomDefault::Fit => ZoomMode::Fit,
                    ZoomDefault::Actual => ZoomMode::Actual,
                };
                s.pan_offset = (0.0, 0.0);
            }
            notify_image_changed(state);
        }
        Err(e) => {
            show_decode_error(state, path, &e);
        }
    }
}

fn show_decode_error(state: &Rc<RefCell<AppState>>, path: &Path, error: &formats::DecodeError) {
    let msg = format!("Could not open {}", path.display());
    let detail = error.to_string();
    eprintln!("{msg}: {detail}");

    let window = state.borrow().window.clone();
    if let Some(win) = window {
        let dialog = gtk4::AlertDialog::builder()
            .message(&msg)
            .detail(&detail)
            .buttons(["OK"])
            .default_button(0)
            .modal(true)
            .build();
        dialog.choose(Some(&win), None::<&gtk4::gio::Cancellable>, |_| {});
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NavigateAction {
    Next,
    Prev,
    First,
    Last,
}

fn pixbuf_from_decoded(decoded: &DecodedImage) -> Pixbuf {
    Pixbuf::from_bytes(
        &glib::Bytes::from(&decoded.pixels),
        gtk4::gdk_pixbuf::Colorspace::Rgb,
        true, // has_alpha (RGBA)
        8,
        decoded.width as i32,
        decoded.height as i32,
        (decoded.width * 4) as i32, // rowstride
    )
}

fn notify_image_changed(state: &Rc<RefCell<AppState>>) {
    let cb = state.borrow().on_image_changed.clone();
    if let Some(cb) = cb {
        cb();
    }
}
