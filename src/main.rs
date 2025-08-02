mod folder;
mod scan;
mod theme;
mod ui;
mod utils;
mod widgets;

use fltk::{enums::Event, prelude::*, *};
use rfd::FileDialog;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use scan::scan_folder_hierarchy;
use theme::*;
use ui::{style_button, update_progress_bar};
use widgets::TreemapWidget;

fn main() {
    let app = app::App::default();

    app::set_background_color(
        CATPPUCCIN_BASE as u8,
        (CATPPUCCIN_BASE >> 8) as u8,
        (CATPPUCCIN_BASE >> 16) as u8,
    );

    let selected_path = Rc::new(RefCell::new(PathBuf::new()));

    let mut wind = window::Window::new(100, 100, 900, 900, "folder-scan");
    wind.set_color(enums::Color::from_u32(CATPPUCCIN_BASE));
    wind.make_resizable(true);
    wind.size_range(400, 400, 0, 0);

    // title text
    let mut title = frame::Frame::new(20, 10, 860, 30, "Folder Scan");
    title.set_label_size(24);
    title.set_label_color(enums::Color::from_u32(CATPPUCCIN_LAVENDER));
    title.set_label_font(enums::Font::HelveticaBold);

    // progress value and bar
    let progress = Rc::new(RefCell::new(frame::Frame::new(20, 80, 860, 30, "")));
    progress.borrow_mut().set_frame(enums::FrameType::FlatBox);

    // status text
    let status_text = Rc::new(RefCell::new(frame::Frame::new(
        20,
        50,
        860,
        30,
        "Select a folder...",
    )));
    status_text
        .borrow_mut()
        .set_label_color(enums::Color::from_u32(CATPPUCCIN_TEXT));
    status_text.borrow_mut().set_align(enums::Align::Center);

    // path chooser button
    let mut button_picker = button::Button::new(285, 125, 140, 40, "Choose Folder");
    style_button(&mut button_picker, CATPPUCCIN_LAVENDER);

    // scan button
    let button_scan = Rc::new(RefCell::new(button::Button::new(475, 125, 140, 40, "Scan")));
    style_button(&mut button_scan.borrow_mut(), CATPPUCCIN_GREEN);
    button_scan
        .borrow_mut()
        .set_color(enums::Color::from_u32(CATPPUCCIN_GREEN));
    button_scan
        .borrow_mut()
        .set_label_color(enums::Color::from_u32(CATPPUCCIN_BASE));
    button_scan.borrow_mut().deactivate();

    // treemap widget
    let treemap = Rc::new(RefCell::new(TreemapWidget::new(20, 180, 860, 700)));

    let treemap_clone = treemap.clone();
    wind.handle(move |_, ev| {
        if ev == Event::Resize {
            treemap_clone.borrow_mut().handle_resize();
        }

        true
    });

    // path select button callback
    {
        let selected_path_picker = Rc::clone(&selected_path);
        let status_text_picker = Rc::clone(&status_text);
        let treemap_clone = Rc::clone(&treemap);
        let progress_bar_clone = Rc::clone(&progress);
        let button_scan_clone = Rc::clone(&button_scan);
        button_picker.set_callback(move |_| {
            if let Some(path) = FileDialog::new().set_directory(".").pick_folder() {
                let text = format!("Selected: {}", path.display());
                status_text_picker.borrow_mut().set_label(&text);
                *selected_path_picker.borrow_mut() = path;
                treemap_clone.borrow_mut().clear();
                button_scan_clone.borrow_mut().activate();
                update_progress_bar(&mut progress_bar_clone.borrow_mut(), 0);
            }
        });
    }

    // scan button callback
    {
        let button_scan_clone = Rc::clone(&button_scan);
        let selected_path_scan = Rc::clone(&selected_path);
        let progress_scan = Rc::clone(&progress);
        let status_text_scan = Rc::clone(&status_text);

        button_scan.borrow_mut().set_callback(move |_| {
            let path = selected_path_scan.borrow().clone();
            if !path.exists() || !path.is_dir() {
                status_text_scan
                    .borrow_mut()
                    .set_label("Please select a valid folder first!");
                return;
            }

            button_scan_clone.borrow_mut().deactivate();
            status_text_scan.borrow_mut().set_label("");

            button_scan_clone.borrow_mut().deactivate();

            // reset progress
            update_progress_bar(&mut progress_scan.borrow_mut(), 0);
            app::flush();

            // Create progress callback that updates UI
            let progress_for_callback = Rc::clone(&progress_scan);
            let status_for_callback = Rc::clone(&status_text_scan);

            let progress_callback = |percentage: i32, message: &str| {
                update_progress_bar(&mut progress_for_callback.borrow_mut(), percentage);
                status_for_callback.borrow_mut().set_label(message);
                app::flush();
            };

            match scan_folder_hierarchy(&path, Some(progress_callback)) {
                Ok(mut tree) => {
                    tree.sort_children();
                    treemap.borrow_mut().set_data(&tree);
                }
                Err(e) => {
                    status_text_scan
                        .borrow_mut()
                        .set_label(&format!("Error: {}", e));
                    update_progress_bar(&mut progress_scan.borrow_mut(), 0);
                }
            }

            button_scan_clone.borrow_mut().activate();
        });
    }

    wind.end();
    wind.show();

    // initialize progress bar
    update_progress_bar(&mut progress.borrow_mut(), 0);

    app.run().unwrap();
}
