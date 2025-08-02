use fltk::{prelude::*, *};

use crate::theme::*;

pub fn style_button(btn: &mut button::Button, color: u32) {
    btn.set_color(enums::Color::from_u32(CATPPUCCIN_SURFACE1));
    btn.set_selection_color(enums::Color::from_u32(color).darker());
    btn.set_label_color(enums::Color::from_u32(CATPPUCCIN_TEXT));
    btn.set_label_font(enums::Font::HelveticaBold);
    btn.set_frame(enums::FrameType::RFlatBox);
}

pub fn update_progress_bar(bar: &mut frame::Frame, percentage: i32) {
    bar.draw(move |b| {
        let total_width = b.w();
        let filled_width = (total_width as f64 * percentage as f64 / 100.0).round() as i32;

        // draw background
        draw::set_draw_color(enums::Color::from_u32(CATPPUCCIN_SURFACE0));
        draw::draw_rectf(b.x(), b.y(), total_width, b.h());

        // draw bar content
        if filled_width > 0 {
            let color = match percentage {
                0..=30 => CATPPUCCIN_RED,
                31..=70 => CATPPUCCIN_PEACH,
                _ => CATPPUCCIN_GREEN,
            };
            draw::set_draw_color(enums::Color::from_u32(color));
            draw::draw_rectf(b.x(), b.y(), filled_width, b.h());
        }

        // draw border
        draw::set_draw_color(enums::Color::from_u32(CATPPUCCIN_SURFACE1));
        draw::draw_rect(b.x(), b.y(), total_width, b.h());
    });

    // Only redraw parent if it exists
    if let Some(mut parent) = bar.parent() {
        parent.redraw();
    }
}
