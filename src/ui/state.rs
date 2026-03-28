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
    pub pan_offset: (f64, f64),
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
            pan_offset: (0.0, 0.0),
            on_image_changed: None,
            on_zoom_changed: None,
            on_list_changed: None,
            on_panels_changed: None,
        }
    }

    pub fn zoom_in(&mut self) {
        let current = self.zoom_factor();
        for &step in ZOOM_STEPS {
            if step > current + 0.001 {
                self.zoom = ZoomMode::Custom(step);
                self.pan_offset = (0.0, 0.0);
                if let Some(cb) = &self.on_zoom_changed {
                    cb();
                }
                return;
            }
        }
    }

    pub fn zoom_out(&mut self) {
        let current = self.zoom_factor();
        for &step in ZOOM_STEPS.iter().rev() {
            if step < current - 0.001 {
                self.zoom = ZoomMode::Custom(step);
                self.pan_offset = (0.0, 0.0);
                if let Some(cb) = &self.on_zoom_changed {
                    cb();
                }
                return;
            }
        }
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
            ZoomMode::Fit => 1.0, // Actual fit factor is computed by the canvas
        }
    }

    pub fn header_title(&self) -> String {
        let filename = self
            .image_list
            .current_path()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "chuckles".to_string());

        if let (Some(idx), len) = (self.image_list.current_index(), self.image_list.len()) {
            if len > 0 {
                return format!("{filename} ({} of {len})", idx + 1);
            }
        }
        filename
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
            eprintln!("Error opening {}: {e}", path.display());
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

/// Check if there are unsaved edits. Returns true if safe to proceed.
/// In the future, this should show a confirmation dialog.
pub fn check_unsaved_edits(state: &Rc<RefCell<AppState>>) -> bool {
    let s = state.borrow();
    if s.has_unsaved_edits {
        // TODO: show a GTK dialog asking to save/discard/cancel
        // For now, discard silently but log a warning
        eprintln!("Warning: discarding unsaved edits");
    }
    true
}

/// Navigate to the next/prev/first/last image.
pub fn navigate(state: &Rc<RefCell<AppState>>, action: NavigateAction) {
    if !check_unsaved_edits(state) {
        return;
    }
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
    let path = {
        let mut s = state.borrow_mut();
        if !s.image_list.go_to(index) {
            return;
        }
        s.image_list.current_path().map(|p| p.to_path_buf())
    };

    if let Some(path) = path {
        load_image_at_path(state, &path);
    }
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
            eprintln!("Error loading {}: {e}", path.display());
        }
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
