use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::{fs, thread};

use crate::folder::FolderNode;
use crate::utils::format_size;

const SCAN_THRESHOLD: u64 = 1024 * 1024; // 1MB 
const SUBDIRECTORY_COUNT_THRESHOLD: usize = 10; // threshold for parallel scanning
const THRESHOLD_FACTOR: f64 = 0.0001; // 0.01% of total size
const MAX_CONCURRENT_THREADS: usize = 8; // limit for concurrent threads

pub fn scan_folder_hierarchy<F>(
    root_path: &Path,
    mut progress_callback: Option<F>,
) -> Result<FolderNode, Box<dyn std::error::Error>>
where
    F: FnMut(i32, &str),
{
    let start_time = std::time::Instant::now();

    let root_name = root_path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("root"))
        .to_string_lossy()
        .to_string();

    println!("Starting scan for: {}", root_path.to_string_lossy());
    let mut root_node = FolderNode::new(root_name, root_path.to_path_buf(), 0);
    let total_size = fast_parallel_scan(&mut root_node, &mut progress_callback)?;
    let threshold = (total_size as f64 * THRESHOLD_FACTOR) as u64;

    println!("Scan completed in {:?}", start_time.elapsed());
    println!(
        "Total directory size: {} bytes ({})",
        total_size,
        format_size(total_size)
    );

    if let Some(callback) = &mut progress_callback {
        callback(
            100,
            &format!(
                "Scanned {} in {:.2} seconds!",
                format_size(total_size),
                start_time.elapsed().as_secs_f64()
            ),
        );
    }

    filter_hierarchy(&mut root_node, threshold);

    Ok(root_node)
}

fn filter_hierarchy(node: &mut FolderNode, threshold: u64) {
    // remove children below threshold
    node.children.retain(|child| child.size >= threshold);

    // recursively filter remaining children
    for child in &mut node.children {
        filter_hierarchy(child, threshold);
    }
}

pub fn calculate_directory_size(dir_path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
    let mut total_size = 0u64;

    fn visit_dir(dir: &Path, total: &mut u64) -> Result<(), Box<dyn std::error::Error>> {
        let entries = fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    *total += metadata.len();
                }
            } else if path.is_dir() {
                visit_dir(&path, total)?;
            }
        }
        Ok(())
    }

    visit_dir(dir_path, &mut total_size)?;
    Ok(total_size)
}

fn fast_parallel_scan<F>(
    parent_node: &mut FolderNode,
    progress_callback: &mut Option<F>,
) -> Result<u64, Box<dyn std::error::Error>>
where
    F: FnMut(i32, &str),
{
    let (tx, rx) = mpsc::channel();
    let path = parent_node.path.clone();

    if let Some(callback) = progress_callback {
        callback(20, &format!("Scanning: {}", path.display()));
    }

    // spawn worker thread for this directory
    thread::spawn(move || {
        let result = scan_directory_fast(&path);
        tx.send(result).unwrap();
    });

    // get results
    let (total_size, children_data) = rx
        .recv()?
        .map_err(|e| -> Box<dyn std::error::Error> { e })?;
    parent_node.size = total_size;

    // process children
    for (child_path, child_size) in children_data {
        let child_name = child_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
            .to_string_lossy()
            .to_string();

        let mut child_node = FolderNode::new(child_name, child_path.clone(), child_size);

        // recursively scan big children
        if child_size > SCAN_THRESHOLD {
            if let Some(callback) = progress_callback {
                callback(80, &format!("Deep scanning: {}", child_path.display()));
            }
            fast_parallel_scan(&mut child_node, progress_callback)?;
        }

        parent_node.add_child(child_node);
    }

    Ok(total_size)
}

type ScanResult = Result<(u64, Vec<(PathBuf, u64)>), Box<dyn std::error::Error + Send>>;

fn scan_directory_fast(dir_path: &Path) -> ScanResult {
    let mut total_size = 0u64;
    let mut children = Vec::new();

    // read directory entries in one go
    let entries: Vec<_> = fs::read_dir(dir_path)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

    // process files first
    let mut file_size = 0u64;
    let mut directories = Vec::new();

    for entry in entries {
        let path = entry.path();
        if path.is_file() {
            if let Ok(metadata) = entry.metadata() {
                file_size += metadata.len();
            }
            // TODO: set progress bar based on current child vs total
        } else if path.is_dir() {
            directories.push(path);
        }
    }

    total_size += file_size;

    // process directories (using multiple threads if above threshold)
    if directories.len() > SUBDIRECTORY_COUNT_THRESHOLD {
        // use thread pool
        let (tx, rx) = mpsc::channel();
        let mut handles = Vec::new();

        for dir in directories.into_iter().take(MAX_CONCURRENT_THREADS) {
            // limit concurrent threads
            let tx = tx.clone();
            let handle = thread::spawn(move || {
                let size = calculate_directory_size(&dir).unwrap_or(0);
                tx.send((dir, size)).unwrap();
            });
            handles.push(handle);
        }

        drop(tx); // close sender

        // collect results
        while let Ok((dir, size)) = rx.recv() {
            total_size += size;
            children.push((dir, size));
        }

        // wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
    } else {
        // sequential for few directories
        for dir in directories {
            let size = calculate_directory_size(&dir).unwrap_or(0);
            total_size += size;
            children.push((dir, size));
        }
    }

    Ok((total_size, children))
}
