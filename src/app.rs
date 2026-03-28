use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};

const APP_ID: &str = "io.github.chuckles";

pub fn run() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    // Pass through CLI args so GTK can consume its own flags
    // and we get the file path from the remaining args.
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("chuckles")
        .default_width(1024)
        .default_height(768)
        .build();

    window.present();
}
