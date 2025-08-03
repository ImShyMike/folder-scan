mod folder;
mod scan;
mod theme;
mod ui;
mod utils;
mod widgets;

use fltk::{enums, prelude::*, *};
use rfd::FileDialog;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::env;

use scan::scan_folder_hierarchy;
use theme::*;
use ui::{style_button, update_progress_bar};
use widgets::TreemapWidget;

struct AppState {
    selected_path: PathBuf,
    progress: frame::Frame,
    status_text: frame::Frame,
    treemap: TreemapWidget,
    scan_button: button::Button,
}

fn main() {
    let app = app::App::default();

    app::set_background_color(
        CATPPUCCIN_BASE as u8,
        (CATPPUCCIN_BASE >> 8) as u8,
        (CATPPUCCIN_BASE >> 16) as u8,
    );

    let mut wind = window::Window::new(100, 100, 900, 900, "folder-scan");
    wind.set_color(enums::Color::from_u32(CATPPUCCIN_BASE));
    wind.make_resizable(true);
    wind.size_range(400, 400, 0, 0);

    // create shared state
    let app_state = Rc::new(RefCell::new(AppState {
        selected_path: PathBuf::new(),
        progress: frame::Frame::new(20, 80, 860, 30, ""),
        status_text: frame::Frame::new(20, 50, 860, 30, "Select a folder..."),
        treemap: TreemapWidget::new(20, 180, 860, 700),
        scan_button: button::Button::new(475, 125, 140, 40, "Scan"),
    }));

    // progress bar styling
    app_state.borrow_mut().progress.set_frame(enums::FrameType::FlatBox);

    // status text styling
    app_state.borrow_mut().status_text.set_label_color(enums::Color::from_u32(CATPPUCCIN_TEXT));
    app_state.borrow_mut().status_text.set_align(enums::Align::Center);

    // scan button styling
    style_button(&mut app_state.borrow_mut().scan_button, CATPPUCCIN_GREEN);
    app_state.borrow_mut().scan_button.set_color(enums::Color::from_u32(CATPPUCCIN_GREEN));
    app_state.borrow_mut().scan_button.set_label_color(enums::Color::from_u32(CATPPUCCIN_BASE));
    app_state.borrow_mut().scan_button.deactivate();

    // title text
    let mut title = frame::Frame::new(20, 10, 860, 30, "Folder Scan");
    title.set_label_size(24);
    title.set_label_color(enums::Color::from_u32(CATPPUCCIN_LAVENDER));
    title.set_label_font(enums::Font::HelveticaBold);

    // path chooser button
    let mut folder_select_button = button::Button::new(285, 125, 140, 40, "Choose Folder");
    style_button(&mut folder_select_button, CATPPUCCIN_LAVENDER);

    // Use weak reference for resize handler
    let treemap_weak = Rc::downgrade(&app_state);
    wind.handle(move |_, ev| {
        if ev == enums::Event::Resize {
            if let Some(state) = treemap_weak.upgrade() {
                state.borrow_mut().treemap.handle_resize();
            }
        }
        true
    });

    // path select button callback
    {
        let state_weak = Rc::downgrade(&app_state);
        folder_select_button.set_callback(move |_| {
            if let Some(state) = state_weak.upgrade() {
                handle_folder_select(&mut state.borrow_mut());
            }
        });
    }

    // scan button callback
    {
        let state_weak = Rc::downgrade(&app_state);
        app_state.borrow_mut().scan_button.set_callback(move |_| {
            if let Some(state) = state_weak.upgrade() {
                handle_scan_button(&mut state.borrow_mut());
            }
        });
    }

    wind.end();
    wind.show();

    // initialize progress bar
    update_progress_bar(&mut app_state.borrow_mut().progress, 0);

    // handle command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let path = &args[1];

        // set initial path if provided
        if let Ok(selected_path) = PathBuf::from(path).canonicalize() {
            let mut state = app_state.borrow_mut();
            state.selected_path = selected_path;
            let text = format!("Selected: {}", state.selected_path.display());
            state.status_text.set_label(&text);
            state.scan_button.activate();
            
            // schedule the scan to happen after the UI loop starts
            let state_weak = Rc::downgrade(&app_state);
            app::add_timeout3(0.1, move |_| {
                if let Some(state) = state_weak.upgrade() {
                    handle_scan_button(&mut state.borrow_mut());
                }
            });
        }
    }

    app.run().unwrap();
}

fn handle_folder_select(state: &mut AppState) {
    if let Some(path) = FileDialog::new().set_directory(".").pick_folder() {
        let text = format!("Selected: {}", path.display());
        state.status_text.set_label(&text);
        state.selected_path = path;
        state.treemap.clear();
        state.scan_button.activate();
    }
}

fn handle_scan_button(state: &mut AppState) {
    let path = state.selected_path.clone();
    if !path.exists() || !path.is_dir() {
        state.status_text.set_label("Please select a valid folder first!");
        return;
    }

    state.scan_button.deactivate();
    state.status_text.set_label("");

    // reset progress
    update_progress_bar(&mut state.progress, 0);
    app::flush();

    let progress_callback = |percentage: i32, message: &str| {
        update_progress_bar(&mut state.progress, percentage);
        state.status_text.set_label(message);
        app::flush();
    };

    match scan_folder_hierarchy(&path, Some(progress_callback)) {
        Ok(mut tree) => {
            tree.sort_children();
            state.treemap.set_data(&tree);
        }
        Err(e) => {
            state.status_text.set_label(&format!("Error: {}", e));
            update_progress_bar(&mut state.progress, 0);
        }
    }

    state.scan_button.activate();
}
