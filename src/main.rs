mod folder;
mod scan;
mod theme;
mod ui;
mod utils;

use fltk::{prelude::*, *};
use rfd::FileDialog;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use folder::FolderNode;
use scan::scan_folder_hierarchy;
use theme::*;
use ui::{style_button, update_progress_bar};
use utils::format_tree_summary;

fn main() {
    let app = app::App::default();

    app::set_background_color(
        CATPPUCCIN_BASE as u8,
        (CATPPUCCIN_BASE >> 8) as u8,
        (CATPPUCCIN_BASE >> 16) as u8,
    );

    let selected_path = Rc::new(RefCell::new(PathBuf::new()));
    let folder_tree = Rc::new(RefCell::new(Option::<FolderNode>::None));

    let mut wind = window::Window::new(100, 100, 500, 650, "folder-scan");
    wind.set_color(enums::Color::from_u32(CATPPUCCIN_BASE));
    wind.make_resizable(true);
    wind.size_range(400, 400, 0, 0);

    // title text
    let mut title = frame::Frame::new(20, 20, 460, 30, "Folder Scan");
    title.set_label_size(18);
    title.set_label_color(enums::Color::from_u32(CATPPUCCIN_LAVENDER));
    title.set_label_font(enums::Font::HelveticaBold);

    // progress value and bar
    let progress_val = Rc::new(RefCell::new(0));
    let progress = Rc::new(RefCell::new(frame::Frame::new(20, 70, 460, 30, "")));
    progress.borrow_mut().set_frame(enums::FrameType::FlatBox);
    progress
        .borrow_mut()
        .set_color(enums::Color::from_u32(CATPPUCCIN_SURFACE0));

    // status text
    let status_text = Rc::new(RefCell::new(frame::Frame::new(
        20,
        110,
        460,
        30,
        "Select a folder...",
    )));
    status_text
        .borrow_mut()
        .set_label_color(enums::Color::from_u32(CATPPUCCIN_TEXT));
    status_text
        .borrow_mut()
        .set_align(enums::Align::Center);

    // path chooser button
    let mut button_picker = button::Button::new(90, 160, 140, 40, "Choose Folder");
    style_button(&mut button_picker, CATPPUCCIN_LAVENDER);

    // scan button
    let button_scan = Rc::new(RefCell::new(button::Button::new(
        270,
        160,
        140,
        40,
        "Scan Sizes",
    )));
    style_button(&mut button_scan.borrow_mut(), CATPPUCCIN_GREEN);

    // results display area
    let results_display = Rc::new(RefCell::new(text::TextDisplay::new(20, 220, 460, 400, "")));
    results_display
        .borrow_mut()
        .set_color(enums::Color::from_u32(CATPPUCCIN_SURFACE0));
    results_display
        .borrow_mut()
        .set_text_color(enums::Color::from_u32(CATPPUCCIN_TEXT));
    results_display
        .borrow_mut()
        .set_text_font(enums::Font::Courier);
    results_display.borrow_mut().set_text_size(12);

    // path select button callback
    {
        let selected_path_picker = Rc::clone(&selected_path);
        let status_text_picker = Rc::clone(&status_text);
        button_picker.set_callback(move |_| {
            if let Some(path) = FileDialog::new().set_directory(".").pick_folder() {
                let text = format!("Selected: {}", path.display());
                status_text_picker.borrow_mut().set_label(&text);
                *selected_path_picker.borrow_mut() = path;
            }
        });
    }

    // scan button callback
    {
        let button_scan_clone = Rc::clone(&button_scan);
        let selected_path_scan = Rc::clone(&selected_path);
        let folder_tree_scan = Rc::clone(&folder_tree);
        let progress_scan = Rc::clone(&progress);
        let progress_val_scan = Rc::clone(&progress_val);
        let status_text_scan = Rc::clone(&status_text);
        let results_display_scan = Rc::clone(&results_display);

        button_scan.borrow_mut().set_callback(move |_| {
            let path = selected_path_scan.borrow().clone();
            if !path.exists() || !path.is_dir() {
                status_text_scan
                    .borrow_mut()
                    .set_label("Please select a valid folder first!");
                return;
            }

            button_scan_clone.borrow_mut().deactivate();
            status_text_scan
                .borrow_mut()
                .set_label("Scanning folder sizes...");

            // reset progress
            *progress_val_scan.borrow_mut() = 0;
            update_progress_bar(&mut progress_scan.borrow_mut(), 0);
            app::flush();

            match scan_folder_hierarchy(&path) {
                Ok(mut tree) => {
                    tree.sort_children();
                    let summary = format_tree_summary(&tree);

                    results_display_scan
                        .borrow_mut()
                        .set_buffer(text::TextBuffer::default());
                    results_display_scan
                        .borrow_mut()
                        .buffer()
                        .unwrap()
                        .set_text(&summary);

                    *folder_tree_scan.borrow_mut() = Some(tree);
                    *progress_val_scan.borrow_mut() = 100;

                    update_progress_bar(&mut progress_scan.borrow_mut(), 100);
                    status_text_scan
                        .borrow_mut()
                        .set_label("Scan completed successfully!");
                }
                Err(e) => {
                    status_text_scan
                        .borrow_mut()
                        .set_label(&format!("Error: {}", e));
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
