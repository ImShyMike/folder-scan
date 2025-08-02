use fltk::draw::*;
use fltk::enums::*;
use fltk::{prelude::*, *};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use open;

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
    size: u64,
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

            // check if the text fits using proper text measurement
            let (text_width, _text_height) = measure(&rect.name, false);

            if text_width <= rect.width - 8 {
                // draw text in the bottom right of the rectangle
                let tx = rect.x + rect.width - 4;
                let ty = rect.y + rect.height - 2;
                draw_text2(&rect.name, tx, ty, 0, 0, Align::BottomRight);
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
                        open::that(rect.path.clone()).unwrap_or_else(|_| {
                            eprintln!("Failed to open path: {}", rect.path.display());
                        });
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
            size: root.size,
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

        let children_total_size: u64 = valid_children.iter().map(|child| child.size).sum();

        // make children only take up space proportional to their size relative to the parent
        let children_area_proportion = children_total_size as f64 / folder.size as f64;

        // add padding to prevent overlap
        let padding = 4;
        let padded_area = TreemapArea {
            x: area.x + padding,
            y: area.y + padding,
            width: (area.width - 2 * padding).max(1),
            height: (area.height - 2 * padding).max(1),
            size: area.size,
        };

        // calculate the actual area children should occupy
        let children_pixel_area =
            (padded_area.width * padded_area.height) as f64 * children_area_proportion;

        // determine child width based on aspect ratio
        let chosen_size = if padded_area.width >= padded_area.height {
            padded_area.height
        } else {
            padded_area.width
        } as f64;

        let child_width = (children_pixel_area / chosen_size) as i32;

        let child_area = TreemapArea {
            x: padded_area.x,
            y: padded_area.y,
            width: child_width.min(padded_area.width),
            height: padded_area.height,
            size: children_total_size,
        };

        let child_areas = self.squarify_layout(&valid_children, child_area);

        // recursively layout children
        for (child, child_area) in valid_children.iter().zip(child_areas.iter()) {
            self.layout_folder(child, *child_area, depth + 1, rects);
        }
    }

    fn squarify_layout(&self, children: &[&FolderNode], area: TreemapArea) -> Vec<TreemapArea> {
        if children.is_empty() {
            return Vec::new();
        }

        // calculate total size of all children
        let total_size: u64 = children.iter().map(|child| child.size).sum();
        if total_size == 0 {
            return Vec::new();
        }

        // get all of the children's sizes
        let folder_sizes: Vec<u64> = children.iter().map(|child| child.size).collect();

        let mut areas = Vec::new();
        let mut remaining_area = area;
        let mut start_idx = 0;

        while start_idx < children.len() {
            // calculate remaining total size for items that haven't been placed yet
            let remaining_total_size: u64 = folder_sizes[start_idx..].iter().sum();

            let row_end = self.find_best_row(
                &folder_sizes[start_idx..],
                remaining_area,
                remaining_total_size,
            );
            let end_idx = start_idx + row_end;

            // calculate areas for this row
            let row_areas = self.layout_row(
                &folder_sizes[start_idx..end_idx],
                remaining_area,
                remaining_total_size,
            );
            areas.extend(row_areas);

            // update remaining area for next iteration
            remaining_area = self.get_remaining_area(
                remaining_area,
                &folder_sizes[start_idx..end_idx],
                remaining_total_size,
            );
            start_idx = end_idx;
        }

        areas
    }

    fn find_best_row(&self, sizes: &[u64], area: TreemapArea, total_size: u64) -> usize {
        if sizes.is_empty() {
            return 0;
        }

        let mut best_count = 1;
        let mut best_aspect_ratio = f64::INFINITY;

        for count in 1..=sizes.len() {
            let row_sum: u64 = sizes[..count].iter().sum();
            let min_size = sizes[..count].iter().min().unwrap();
            let max_size = sizes[..count].iter().max().unwrap();

            let worst_aspect_ratio =
                self.calculate_worst_ratio(row_sum, *min_size, *max_size, area, total_size);

            if worst_aspect_ratio < best_aspect_ratio {
                best_aspect_ratio = worst_aspect_ratio;
                best_count = count;
            } else {
                break;
            }
        }

        best_count
    }

    fn calculate_worst_ratio(
        &self,
        row_sum: u64,
        min_size: u64,
        max_size: u64,
        area: TreemapArea,
        total_size: u64,
    ) -> f64 {
        let is_horizontal = area.width >= area.height;

        // calculate the area proportions
        let area_total = (area.width * area.height) as f64;
        let row_area_proportion = row_sum as f64 / total_size as f64;
        let row_pixel_area = area_total * row_area_proportion;

        let side_length = if is_horizontal {
            area.height
        } else {
            area.width
        } as f64;

        // laying out horizontally
        let row_width = row_pixel_area / side_length;
        let min_pixel_area = (min_size as f64 / total_size as f64) * area_total;
        let max_pixel_area = (max_size as f64 / total_size as f64) * area_total;

        let min_opposite_side_length = min_pixel_area / row_width;
        let max_opposite_side_length = max_pixel_area / row_width;

        let aspect1 = row_width / min_opposite_side_length;
        let aspect2 = max_opposite_side_length / row_width;
        aspect1.max(aspect2)
    }

    fn layout_row(&self, sizes: &[u64], area: TreemapArea, total_size: u64) -> Vec<TreemapArea> {
        let mut result = Vec::new();

        if sizes.is_empty() {
            return result;
        }

        let row_sum: u64 = sizes.iter().sum();
        let is_horizontal = area.width >= area.height;

        // calculate total area for this row based on proportional sizes
        let area_total = (area.width * area.height) as f64;
        let row_area_proportion = row_sum as f64 / total_size as f64;
        let row_pixel_area = area_total * row_area_proportion;

        if is_horizontal {
            // layout horizontally
            let row_width = row_pixel_area / area.height as f64;
            let mut current_y = area.y;

            for &size in sizes {
                let size_proportion = size as f64 / row_sum as f64;
                let rect_height = (size_proportion * area.height as f64) as i32;
                let rect_height = rect_height.min(area.height - (current_y - area.y));

                if rect_height > 0 {
                    result.push(TreemapArea {
                        x: area.x,
                        y: current_y,
                        width: row_width as i32,
                        height: rect_height,
                        size,
                    });
                    current_y += rect_height;
                }
            }
        } else {
            // layout vertically
            let row_height = row_pixel_area / area.width as f64;
            let mut current_x = area.x;

            for &size in sizes {
                let size_proportion = size as f64 / row_sum as f64;
                let rect_width = (size_proportion * area.width as f64) as i32;
                let rect_width = rect_width.min(area.width - (current_x - area.x));

                if rect_width > 0 {
                    result.push(TreemapArea {
                        x: current_x,
                        y: area.y,
                        width: rect_width,
                        height: row_height as i32,
                        size,
                    });
                    current_x += rect_width;
                }
            }
        }

        result
    }

    fn get_remaining_area(
        &self,
        area: TreemapArea,
        placed_sizes: &[u64],
        total_size: u64,
    ) -> TreemapArea {
        let row_sum: u64 = placed_sizes.iter().sum();
        let is_horizontal = area.width >= area.height;

        // calculate how much area was used based on folder sizes
        let area_total = (area.width * area.height) as f64;
        let row_area_proportion = row_sum as f64 / total_size as f64;
        let row_pixel_area = area_total * row_area_proportion;

        if is_horizontal {
            let used_width = (row_pixel_area / area.height as f64) as i32;
            TreemapArea {
                x: area.x + used_width,
                y: area.y,
                width: (area.width - used_width).max(0),
                height: area.height,
                size: area.size - row_sum,
            }
        } else {
            let used_height = (row_pixel_area / area.width as f64) as i32;
            TreemapArea {
                x: area.x,
                y: area.y + used_height,
                width: area.width,
                height: (area.height - used_height).max(0),
                size: area.size - row_sum,
            }
        }
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
