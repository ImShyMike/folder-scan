use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FolderNode {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub children: Vec<FolderNode>,
}

impl FolderNode {
    #[inline]
    pub fn new(name: String, path: PathBuf, size: u64) -> Self {
        Self {
            name,
            path,
            size,
            children: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn add_child(&mut self, child: FolderNode) {
        self.children.push(child);
    }

    // Sort children by size (largest first)
    pub fn sort_children(&mut self) {
        self.children.sort_by(|a, b| b.size.cmp(&a.size));
        for child in &mut self.children {
            child.sort_children();
        }
    }
}
