use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::ApplicationWindow;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;

use crate::config::ScrollWheelMode;
use crate::ui::state::{self, AppState, NavigateAction, ZoomMode};

pub fn setup_mouse_handlers(window: &ApplicationWindow, state: &Rc<RefCell<AppState>>) {
    setup_scroll_handler(window, state);
    setup_drag_handler(window, state);
    setup_double_click_handler(window, state);
}

fn setup_scroll_handler(window: &ApplicationWindow, state: &Rc<RefCell<AppState>>) {
    let controller = gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);

    let state = state.clone();
    controller.connect_scroll(move |ctrl, _dx, dy| {
        let ctrl_held = ctrl
            .current_event()
            .map(|e| e.modifier_state().contains(gdk::ModifierType::CONTROL_MASK))
            .unwrap_or(false);

        if ctrl_held {
            // Ctrl+scroll zooms centered on the mouse cursor position.
            // EventControllerScroll doesn't expose cursor coords directly,
            // so we use the last known motion position stored in state,
            // falling back to canvas center.
            let (canvas_w, canvas_h) = get_widget_size(ctrl);
            let (cx, cy) = {
                let s = state.borrow();
                s.last_mouse_pos.unwrap_or((canvas_w / 2.0, canvas_h / 2.0))
            };

            let mut s = state.borrow_mut();
            s.zoom_at_point(dy < 0.0, cx, cy, canvas_w, canvas_h);
        } else {
            let scroll_mode = state.borrow().scroll_wheel;
            match scroll_mode {
                ScrollWheelMode::Navigate => {
                    if dy > 0.0 {
                        state::navigate(&state, NavigateAction::Next);
                    } else {
                        state::navigate(&state, NavigateAction::Prev);
                    }
                }
                ScrollWheelMode::Zoom => {
                    let mut s = state.borrow_mut();
                    if dy < 0.0 {
                        s.zoom_in();
                    } else {
                        s.zoom_out();
                    }
                }
            }
        }

        glib::Propagation::Stop
    });

    window.add_controller(controller);
}

fn get_widget_size(ctrl: &gtk4::EventControllerScroll) -> (f64, f64) {
    ctrl.widget()
        .map(|w| (w.width() as f64, w.height() as f64))
        .unwrap_or((1.0, 1.0))
}

fn setup_drag_handler(window: &ApplicationWindow, state: &Rc<RefCell<AppState>>) {
    let drag = gtk4::GestureDrag::new();

    let start_offset: Rc<Cell<(f64, f64)>> = Rc::new(Cell::new((0.0, 0.0)));

    let state_start = state.clone();
    let start_ref = start_offset.clone();
    drag.connect_drag_begin(move |_, _, _| {
        let s = state_start.borrow();
        start_ref.set(s.pan_offset);
    });

    let state_update = state.clone();
    let update_ref = start_offset.clone();
    drag.connect_drag_update(move |_, dx, dy| {
        let mut s = state_update.borrow_mut();
        // Only pan when zoomed beyond canvas bounds
        if matches!(s.zoom, ZoomMode::Fit) {
            return;
        }
        let (sx, sy) = update_ref.get();
        s.pan_offset = (sx + dx, sy + dy);
        if let Some(cb) = &s.on_zoom_changed {
            cb();
        }
    });

    window.add_controller(drag);
}

fn setup_double_click_handler(window: &ApplicationWindow, state: &Rc<RefCell<AppState>>) {
    let gesture = gtk4::GestureClick::new();
    gesture.set_button(1); // Left click

    let state = state.clone();
    gesture.connect_released(move |_gesture, n_press, _, _| {
        if n_press == 2 {
            let mut s = state.borrow_mut();
            if !s.is_fullscreen {
                s.is_fullscreen = true;
                if let Some(cb) = &s.on_panels_changed {
                    cb();
                }
            }
        }
    });

    window.add_controller(gesture);
}
