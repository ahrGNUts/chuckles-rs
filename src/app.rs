use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};

use crate::config;
use crate::ui;

const APP_ID: &str = "io.github.chuckles";

pub fn run() {
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN | gio::ApplicationFlags::NON_UNIQUE)
        .build();

    let config = Rc::new(RefCell::new(config::load()));

    let config_for_activate = config.clone();
    app.connect_activate(move |app| {
        build_ui(app, &config_for_activate, None);
    });

    let config_for_open = config.clone();
    app.connect_open(move |app, files, _hint| {
        if let Some(file) = files.first() {
            let path = file.path();
            build_ui(app, &config_for_open, path);
        }
    });

    app.run();
}

fn build_ui(
    app: &Application,
    config: &Rc<RefCell<config::AppConfig>>,
    file_path: Option<PathBuf>,
) {
    let conf = config.borrow();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("chuckles")
        .default_width(conf.window.width)
        .default_height(conf.window.height)
        .build();

    let state = Rc::new(RefCell::new(ui::state::AppState::new(
        conf.sort_mode,
        conf.sort_direction,
        conf.default_zoom,
        conf.scroll_wheel,
        conf.background_color.clone(),
        conf.window.sidebar_visible,
        conf.window.thumbnail_strip_visible,
    )));
    drop(conf);

    state.borrow_mut().window = Some(window.clone());

    let main_layout = ui::layout::build_main_layout(&window, &state);
    window.set_child(Some(&main_layout));

    ui::keyboard::setup_key_handler(&window, &state);
    ui::mouse::setup_mouse_handlers(&window, &state);

    let config_for_close = config.clone();
    let state_for_close = state.clone();
    window.connect_close_request(move |win| {
        let s = state_for_close.borrow();
        let mut conf = config_for_close.borrow_mut();
        let (width, height) = win.default_size();
        conf.window.width = width;
        conf.window.height = height;
        conf.window.sidebar_visible = s.sidebar_visible;
        conf.window.thumbnail_strip_visible = s.thumbnail_strip_visible;
        drop(s);
        let _ = config::save(&conf);
        glib::Propagation::Proceed
    });

    window.present();

    match file_path {
        Some(path) => ui::state::open_file(&state, &path),
        None => open_file_chooser(&window, &state),
    }
}

fn open_file_chooser(window: &ApplicationWindow, state: &Rc<RefCell<ui::state::AppState>>) {
    let dialog = gtk4::FileDialog::builder()
        .title("Open Image")
        .modal(true)
        .build();

    let filter = gtk4::FileFilter::new();
    filter.set_name(Some("Image files"));
    filter.add_mime_type("image/jpeg");
    filter.add_mime_type("image/png");
    filter.add_mime_type("image/gif");
    filter.add_mime_type("image/bmp");
    filter.add_mime_type("image/webp");
    filter.add_mime_type("image/tiff");
    filter.add_mime_type("image/jxl");
    filter.add_mime_type("image/svg+xml");

    let filters = gio::ListStore::new::<gtk4::FileFilter>();
    filters.append(&filter);
    dialog.set_filters(Some(&filters));

    let state = state.clone();
    dialog.open(Some(window), None::<&gio::Cancellable>, move |result| {
        if let Ok(file) = result {
            let Some(path) = file.path() else { return };
            ui::state::open_file(&state, &path);
        }
    });
}
