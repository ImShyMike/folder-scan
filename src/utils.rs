use crate::folder::FolderNode;

const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

pub fn format_tree_summary(node: &FolderNode) -> String {
    let mut result = String::new();
    format_tree_recursive(node, &mut result, 0, true);
    result
}

pub fn format_size(bytes: u64) -> String {
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

fn format_tree_recursive(node: &FolderNode, result: &mut String, depth: usize, is_last: bool) {
    let prefix = if depth == 0 {
        "📁 "
    } else {
        if is_last {
            "└── 📁 "
        } else {
            "├── 📁 "
        }
    };

    let indent = "    ".repeat(depth.saturating_sub(1));
    result.push_str(&format!(
        "{}{}{} ({})\n",
        indent,
        prefix,
        node.name,
        format_size(node.size)
    ));

    for (i, child) in node.children.iter().enumerate() {
        let is_child_last = i == node.children.len() - 1;
        format_tree_recursive(child, result, depth + 1, is_child_last);
    }
}
