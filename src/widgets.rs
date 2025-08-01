use fltk::draw::*;
use fltk::enums::*;
use fltk::{prelude::*, *};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::folder::FolderNode;
use crate::theme::*;
use crate::utils::format_size;

#[derive(Clone, Debug)]
struct TreemapRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    name: String,
    path: PathBuf,
    size: u64,
    depth: u32,
    color: Color,
}

#[derive(Copy, Clone, Debug)]
struct TreemapArea {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

struct TreemapData {
    rects: Vec<TreemapRect>,
    hovered_rect: Option<usize>,
    root_node: Option<FolderNode>,
}

pub struct TreemapWidget {
    widget: widget::Widget,
    data: Rc<RefCell<TreemapData>>,
}

impl TreemapWidget {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let mut widget = widget::Widget::default().with_size(w, h).with_pos(x, y);

        widget.set_trigger(CallbackTrigger::Never);

        let data = Rc::new(RefCell::new(TreemapData {
            rects: Vec::new(),
            hovered_rect: None,
            root_node: None,
        }));

        let data_draw = data.clone();
        let data_handle = data.clone();

        widget.draw(move |w| {
            Self::draw_callback(w, &data_draw);
        });

        widget.handle(move |w, event| Self::handle_callback(w, event, &data_handle));

        Self { widget, data }
    }

    fn draw_callback(w: &mut widget::Widget, data: &Rc<RefCell<TreemapData>>) {
        let data = data.borrow();

        // clear background
        draw_rect_fill(
            w.x(),
            w.y(),
            w.width(),
            w.height(),
            Color::from_hex(CATPPUCCIN_SURFACE1),
        );

        // sort rectangles by depth
        let mut sorted_rects: Vec<(usize, &TreemapRect)> = data.rects.iter().enumerate().collect();
        sorted_rects.sort_by(|a, b| a.1.depth.cmp(&b.1.depth));

        // get all rectangles that should be highlighted
        let highlighted_rects = Self::get_highlighted_rects(&data.rects, data.hovered_rect);

        // draw all rectangles in depth order
        for (i, rect) in sorted_rects {
            Self::draw_rect(rect, highlighted_rects.contains(&i));
        }

        // draw tooltip for hovered rectangle
        if let Some(i) = data.hovered_rect {
            if let Some(rect) = data.rects.get(i) {
                Self::draw_tooltip(w, rect);
            }
        }
    }

    fn draw_rect(rect: &TreemapRect, is_hovered: bool) {
        // skip drawing very small rectangles
        if rect.width < 2 || rect.height < 2 {
            return;
        }

        let color = if is_hovered {
            rect.color.darker()
        } else {
            rect.color
        };

        let border_color = Color::from_hex(CATPPUCCIN_BASE);

        // fill the rectangle
        draw_rect_fill(rect.x, rect.y, rect.width, rect.height, color);

        // draw a border
        draw_rect_with_color(rect.x, rect.y, rect.width, rect.height, border_color);

        // draw text if rectangle is large enough
        if rect.width > 50 && rect.height > 25 {
            if is_hovered {
                set_draw_color(Color::from_hex(CATPPUCCIN_TEXT).lighter());
            } else {
                set_draw_color(Color::from_hex(CATPPUCCIN_BASE));
            }

            // use appropriate font size based on rectangle size
            let font_size = if rect.width > 100 && rect.height > 50 {
                12
            } else if rect.width > 75 && rect.height > 35 {
                10
            } else {
                8
            };
            set_font(Font::Helvetica, font_size);

            // draw folder name with proper clipping
            let text_x = rect.x + 4;
            let text_y = rect.y + font_size + 4;

            // ensure text stays within rectangle bounds
            if text_x + 10 < rect.x + rect.width && text_y < rect.y + rect.height {
                draw_text2(&rect.name, text_x, text_y, rect.width - 8, 0, Align::Left);
            }
        }
    }

    fn get_highlighted_rects(
        rects: &[TreemapRect],
        hovered_rect: Option<usize>,
    ) -> std::collections::HashSet<usize> {
        let mut highlighted = std::collections::HashSet::new();

        if let Some(hovered_idx) = hovered_rect {
            if let Some(hovered) = rects.get(hovered_idx) {
                // add the hovered rectangle itself
                highlighted.insert(hovered_idx);

                // find all children
                for (i, rect) in rects.iter().enumerate() {
                    if rect.depth > hovered.depth && Self::is_rect_inside(rect, hovered) {
                        highlighted.insert(i);
                    }
                }
            }
        }

        highlighted
    }

    fn is_rect_inside(inner: &TreemapRect, outer: &TreemapRect) -> bool {
        inner.x >= outer.x
            && inner.y >= outer.y
            && inner.x + inner.width <= outer.x + outer.width
            && inner.y + inner.height <= outer.y + outer.height
    }

    fn draw_tooltip(w: &widget::Widget, rect: &TreemapRect) {
        let size_formatted = format_size(rect.size);
        let tooltip_text = format!(
            "{}\nPath: {}\nSize: {}",
            rect.name,
            rect.path.display(),
            size_formatted
        );

        // get mouse position
        let mouse_x = app::event_x();
        let mouse_y = app::event_y();

        // calculate tooltip position
        let tooltip_x = mouse_x + 15;
        let tooltip_y = mouse_y - 10;

        // calculate text dimensions for tooltip box size
        let lines: Vec<&str> = tooltip_text.split('\n').collect();

        // font size estimation
        set_font(Font::Helvetica, 11);
        let char_width = 7;
        let line_height = 15;

        let max_line_width = lines
            .iter()
            .map(|line| (line.len() as i32) * char_width)
            .max()
            .unwrap_or(100);

        let box_width = max_line_width + 20;
        let box_height = (lines.len() as i32) * line_height + 10;

        // adjust position if tooltip would go off screen
        let final_x = if tooltip_x + box_width > w.x() + w.width() {
            mouse_x - box_width - 15
        } else {
            tooltip_x
        };

        let final_y = if tooltip_y - box_height < w.y() {
            mouse_y + 20
        } else {
            tooltip_y - box_height
        };

        // draw tooltip background with shadow
        draw_rect_fill(
            final_x + 2,
            final_y + 2,
            box_width,
            box_height,
            Color::from_rgba_tuple((0, 0, 0, 128)),
        );
        draw_rect_fill(
            final_x,
            final_y,
            box_width,
            box_height,
            Color::from_hex(CATPPUCCIN_SURFACE0),
        );

        // draw border
        draw_rect_with_color(
            final_x,
            final_y,
            box_width,
            box_height,
            Color::from_hex(CATPPUCCIN_OVERLAY0),
        );

        // draw tooltip text
        set_draw_color(Color::from_hex(CATPPUCCIN_TEXT));
        set_font(Font::Helvetica, 11);

        for (i, line) in lines.iter().enumerate() {
            draw_text2(
                line,
                final_x + 10,
                final_y + 15 + (i as i32 * line_height),
                0,
                0,
                Align::Left,
            );
        }
    }

    fn handle_callback(
        w: &mut widget::Widget,
        event: Event,
        data: &Rc<RefCell<TreemapData>>,
    ) -> bool {
        match event {
            Event::Move => {
                let mouse_x = app::event_x();
                let mouse_y = app::event_y();

                // check if mouse is within widget bounds
                if mouse_x < w.x()
                    || mouse_x >= w.x() + w.width()
                    || mouse_y < w.y()
                    || mouse_y >= w.y() + w.height()
                {
                    let mut data_mut = data.borrow_mut();
                    if data_mut.hovered_rect.is_some() {
                        data_mut.hovered_rect = None;
                        w.redraw();
                    }
                    return true;
                }

                let mut data_mut = data.borrow_mut();

                // find the smallest rectangle that contains the mouse
                let mut best_rect = None;
                let mut best_area = i32::MAX;

                for (i, rect) in data_mut.rects.iter().enumerate() {
                    if mouse_x >= rect.x
                        && mouse_x < rect.x + rect.width
                        && mouse_y >= rect.y
                        && mouse_y < rect.y + rect.height
                    {
                        let area = rect.width * rect.height;

                        // always prefer the smallest containing rectangle
                        if area < best_area {
                            best_rect = Some(i);
                            best_area = area;
                        }
                    }
                }

                if data_mut.hovered_rect != best_rect {
                    data_mut.hovered_rect = best_rect;
                    w.redraw();
                }

                true
            }

            Event::Push => {
                let data_ref = data.borrow();
                if let Some(i) = data_ref.hovered_rect {
                    if let Some(rect) = data_ref.rects.get(i) {
                        println!(
                            "Clicked on: '{}' at '{}' (size: {})",
                            rect.name,
                            rect.path.display(),
                            format_size(rect.size)
                        );
                    }
                }

                true
            }

            Event::Leave => {
                let mut data_mut = data.borrow_mut();
                if data_mut.hovered_rect.is_some() {
                    data_mut.hovered_rect = None;
                    w.redraw();
                }

                true
            }

            Event::Enter => {
                w.take_focus().ok();

                true
            }

            _ => false,
        }
    }

    pub fn handle_resize(&mut self) {
        self.recalculate_layout();
        self.widget.redraw();
    }

    fn recalculate_layout(&self) {
        let mut data = self.data.borrow_mut();
        if let Some(ref root) = data.root_node {
            let rects = self.calculate_hierarchical_treemap(root);
            data.rects = rects;
            data.hovered_rect = None;
        }
    }

    pub fn set_data(&mut self, root: &FolderNode) {
        let rects = self.calculate_hierarchical_treemap(root);

        let mut data = self.data.borrow_mut();
        data.rects = rects;
        data.hovered_rect = None;
        data.root_node = Some(root.clone());

        self.widget.redraw();
    }

    fn calculate_hierarchical_treemap(&self, root: &FolderNode) -> Vec<TreemapRect> {
        let mut rects = Vec::new();

        let area = TreemapArea {
            x: self.widget.x(),
            y: self.widget.y(),
            width: self.widget.width(),
            height: self.widget.height(),
        };

        self.layout_folder(root, area, 0, &mut rects);
        rects
    }

    fn layout_folder(
        &self,
        folder: &FolderNode,
        area: TreemapArea,
        depth: u32,
        rects: &mut Vec<TreemapRect>,
    ) {
        // skip very small areas
        if area.width < 8 || area.height < 8 {
            return;
        }

        let is_leaf = folder.children.is_empty();

        // add rectangle for this folder
        rects.push(TreemapRect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height,
            name: folder.name.clone(),
            path: folder.path.clone(),
            size: folder.size,
            depth,
            color: Self::get_color_for_depth(depth),
        });

        // don't subdivide if it's a leaf or area is too small
        if is_leaf || area.width < 20 || area.height < 20 {
            return;
        }

        // filter out zero-sized children and sort by size
        let mut valid_children: Vec<_> = folder
            .children
            .iter()
            .filter(|child| child.size > 0)
            .collect();

        if valid_children.is_empty() {
            return;
        }

        // sort children by size
        valid_children.sort_by(|a, b| b.size.cmp(&a.size));

        // add padding to prevent overlap
        let padding = 4;
        let padded_area = TreemapArea {
            x: area.x + padding,
            y: area.y + padding,
            width: (area.width - 2 * padding).max(1),
            height: (area.height - 2 * padding).max(1),
        };

        let child_areas = self.squarify_layout(&valid_children, padded_area);

        // recursively layout children
        for (child, child_area) in valid_children.iter().zip(child_areas.iter()) {
            self.layout_folder(child, *child_area, depth + 1, rects);
        }
    }

    fn squarify_layout(&self, children: &[&FolderNode], area: TreemapArea) -> Vec<TreemapArea> {
        if children.is_empty() {
            return Vec::new();
        }

        let total_size: u64 = children.iter().map(|c| c.size).sum();
        if total_size == 0 {
            return vec![area; children.len()];
        }

        // use squarified treemap algorithm for better aspect ratios
        let mut areas = Vec::new();
        let mut start = 0;
        let mut remaining_area = area;

        while start < children.len() {
            let (row_len, row) =
                self.get_best_row_slice(&children[start..], remaining_area, total_size);

            let row_areas = self.layout_row(row, remaining_area, total_size);
            areas.extend(row_areas);

            // update remaining area
            if remaining_area.width >= remaining_area.height {
                // horizontal split
                let row_width = self.calculate_row_width(row, remaining_area, total_size);
                remaining_area = TreemapArea {
                    x: remaining_area.x + row_width,
                    y: remaining_area.y,
                    width: remaining_area.width - row_width,
                    height: remaining_area.height,
                };
            } else {
                // vertical split
                let row_height = self.calculate_row_height(row, remaining_area, total_size);
                remaining_area = TreemapArea {
                    x: remaining_area.x,
                    y: remaining_area.y + row_height,
                    width: remaining_area.width,
                    height: remaining_area.height - row_height,
                };
            }

            start += row_len;
        }

        areas
    }

    fn get_best_row_slice<'a>(
        &self,
        children: &'a [&FolderNode],
        area: TreemapArea,
        total_size: u64,
    ) -> (usize, &'a [&'a FolderNode]) {
        if children.is_empty() {
            return (0, &[]);
        }

        let mut best_len = 1;
        let mut best_ratio = f64::INFINITY;

        for i in 1..=children.len() {
            let row = &children[..i];
            let ratio = self.calculate_worst_ratio(row, area, total_size);

            if ratio < best_ratio {
                best_ratio = ratio;
                best_len = i;
            } else {
                break;
            }
        }

        (best_len, &children[..best_len])
    }

    fn calculate_worst_ratio(
        &self,
        row: &[&FolderNode],
        area: TreemapArea,
        total_size: u64,
    ) -> f64 {
        if row.is_empty() {
            return f64::INFINITY;
        }

        let row_size: u64 = row.iter().map(|c| c.size).sum();
        let row_area =
            (area.width as f64 * area.height as f64) * (row_size as f64 / total_size as f64);

        if row_area <= 0.0 {
            return f64::INFINITY;
        }

        let side_length = if area.width >= area.height {
            row_area / area.height as f64
        } else {
            row_area / area.width as f64
        };

        if side_length <= 0.0 {
            return f64::INFINITY;
        }

        let mut worst_ratio = 0.0f64;
        for &child in row {
            let child_area = row_area * (child.size as f64 / row_size as f64);
            let child_side = child_area / side_length;

            if child_side <= 0.0 {
                continue;
            }

            let ratio = if side_length > child_side {
                side_length / child_side
            } else {
                child_side / side_length
            };

            worst_ratio = worst_ratio.max(ratio);
        }

        worst_ratio
    }

    fn layout_row(
        &self,
        row: &[&FolderNode],
        area: TreemapArea,
        total_size: u64,
    ) -> Vec<TreemapArea> {
        let row_size: u64 = row.iter().map(|c| c.size).sum();
        let mut areas = Vec::new();

        if row_size == 0 {
            return areas;
        }

        let horizontal = area.width >= area.height;
        let row_dimension = if horizontal {
            self.calculate_row_width(row, area, total_size)
        } else {
            self.calculate_row_height(row, area, total_size)
        };

        let mut offset = 0;
        for &child in row {
            let child_ratio = child.size as f64 / row_size as f64;

            let (child_area, child_offset) = if horizontal {
                let child_height = (area.height as f64 * child_ratio) as i32;
                let child_height = child_height.max(1).min(area.height - offset);

                (
                    TreemapArea {
                        x: area.x,
                        y: area.y + offset,
                        width: row_dimension,
                        height: child_height,
                    },
                    child_height,
                )
            } else {
                let child_width = (area.width as f64 * child_ratio) as i32;
                let child_width = child_width.max(1).min(area.width - offset);

                (
                    TreemapArea {
                        x: area.x + offset,
                        y: area.y,
                        width: child_width,
                        height: row_dimension,
                    },
                    child_width,
                )
            };

            areas.push(child_area);
            offset += child_offset;
        }

        areas
    }

    fn calculate_row_width(&self, row: &[&FolderNode], area: TreemapArea, total_size: u64) -> i32 {
        let row_size: u64 = row.iter().map(|c| c.size).sum();
        if total_size == 0 {
            return area.width;
        }
        ((area.width as f64) * (row_size as f64 / total_size as f64)) as i32
    }

    fn calculate_row_height(&self, row: &[&FolderNode], area: TreemapArea, total_size: u64) -> i32 {
        let row_size: u64 = row.iter().map(|c| c.size).sum();
        if total_size == 0 {
            return area.height;
        }
        ((area.height as f64) * (row_size as f64 / total_size as f64)) as i32
    }

    pub fn clear(&mut self) {
        let mut data = self.data.borrow_mut();
        data.rects.clear();
        data.hovered_rect = None;
        data.root_node = None;
        self.widget.redraw();
    }

    fn get_color_for_depth(depth: u32) -> Color {
        match depth % 6 {
            0 => Color::from_hex(CATPPUCCIN_BLUE),
            1 => Color::from_hex(CATPPUCCIN_PEACH),
            2 => Color::from_hex(CATPPUCCIN_GREEN),
            3 => Color::from_hex(CATPPUCCIN_PINK),
            4 => Color::from_hex(CATPPUCCIN_MAUVE),
            _ => Color::from_hex(CATPPUCCIN_YELLOW),
        }
    }
}
