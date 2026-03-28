use std::path::{Path, PathBuf};

use crate::config::{SortDirection, SortMode};
use crate::formats::is_supported_extension;

/// An entry in the directory file list.
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub path: PathBuf,
    pub file_size: u64,
    pub modified: std::time::SystemTime,
}

/// Scan a directory and return entries for all files with supported image extensions.
/// Does not recurse into subdirectories.
pub fn scan_directory(dir: &Path) -> Vec<DirEntry> {
    let read_dir = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };

    read_dir
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if !path.is_file() {
                return None;
            }

            if !is_supported_extension(&path) {
                return None;
            }

            let meta = entry.metadata().ok()?;
            Some(DirEntry {
                path,
                file_size: meta.len(),
                modified: meta.modified().unwrap_or(std::time::UNIX_EPOCH),
            })
        })
        .collect()
}

/// Sort directory entries according to the given mode and direction.
pub fn sort_entries(entries: &mut [DirEntry], mode: SortMode, direction: SortDirection) {
    match mode {
        SortMode::Name => {
            entries.sort_by(|a, b| alphanumeric_sort::compare_path(&a.path, &b.path));
        }
        SortMode::DateModified => {
            entries.sort_by(|a, b| a.modified.cmp(&b.modified));
        }
        SortMode::FileSize => {
            entries.sort_by(|a, b| a.file_size.cmp(&b.file_size));
        }
        SortMode::FileType => {
            entries.sort_by(|a, b| {
                let ext_a = a
                    .path
                    .extension()
                    .map(|e| e.to_ascii_lowercase())
                    .unwrap_or_default();
                let ext_b = b
                    .path
                    .extension()
                    .map(|e| e.to_ascii_lowercase())
                    .unwrap_or_default();
                ext_a
                    .cmp(&ext_b)
                    .then_with(|| alphanumeric_sort::compare_path(&a.path, &b.path))
            });
        }
        SortMode::Dimensions => {
            // Pre-compute areas to avoid O(n log n) file I/O in the comparator.
            let areas: Vec<u64> = entries.iter().map(|e| image_area(&e.path)).collect();
            let mut indices: Vec<usize> = (0..entries.len()).collect();
            indices.sort_by(|&a, &b| areas[a].cmp(&areas[b]));
            let sorted: Vec<DirEntry> = indices.into_iter().map(|i| entries[i].clone()).collect();
            entries.clone_from_slice(&sorted);
        }
    }

    if direction == SortDirection::Descending {
        entries.reverse();
    }
}

/// Get the pixel area of an image by reading only its header.
/// Returns u64::MAX on failure so unknown-dimension files sort to the end.
fn image_area(path: &Path) -> u64 {
    match imagesize::size(path) {
        Ok(dim) => (dim.width as u64) * (dim.height as u64),
        Err(_) => u64::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dir_with_files(names: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for name in names {
            std::fs::write(dir.path().join(name), b"fake").unwrap();
        }
        dir
    }

    #[test]
    fn scan_directory_filters_by_extension() {
        let dir = create_test_dir_with_files(&[
            "photo.jpg",
            "readme.txt",
            "image.png",
            "data.csv",
            "drawing.svg",
        ]);

        let entries = scan_directory(dir.path());

        let names: Vec<&str> = entries
            .iter()
            .map(|e| e.path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert!(names.contains(&"photo.jpg"));
        assert!(names.contains(&"image.png"));
        assert!(names.contains(&"drawing.svg"));
        assert!(!names.contains(&"readme.txt"));
        assert!(!names.contains(&"data.csv"));
    }

    #[test]
    fn scan_directory_excludes_extensionless_files() {
        let dir = create_test_dir_with_files(&["photo", "image.png"]);

        let entries = scan_directory(dir.path());

        assert_eq!(entries.len(), 1);
        assert!(
            entries[0]
                .path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .contains("png")
        );
    }

    #[test]
    fn scan_directory_excludes_directories() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("image.jpg"), b"fake").unwrap();
        std::fs::create_dir(dir.path().join("subdir.jpg")).unwrap();

        let entries = scan_directory(dir.path());

        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn scan_empty_directory_returns_empty() {
        let dir = tempfile::tempdir().unwrap();

        let entries = scan_directory(dir.path());

        assert!(entries.is_empty());
    }

    #[test]
    fn sort_by_name_uses_natural_sort() {
        let dir = create_test_dir_with_files(&["img10.jpg", "img2.jpg", "img1.jpg", "img20.jpg"]);
        let mut entries = scan_directory(dir.path());

        sort_entries(&mut entries, SortMode::Name, SortDirection::Ascending);

        let names: Vec<&str> = entries
            .iter()
            .map(|e| e.path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(
            names,
            vec!["img1.jpg", "img2.jpg", "img10.jpg", "img20.jpg"]
        );
    }

    #[test]
    fn sort_by_name_descending_reverses_order() {
        let dir = create_test_dir_with_files(&["a.jpg", "b.jpg", "c.jpg"]);
        let mut entries = scan_directory(dir.path());

        sort_entries(&mut entries, SortMode::Name, SortDirection::Descending);

        let names: Vec<&str> = entries
            .iter()
            .map(|e| e.path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["c.jpg", "b.jpg", "a.jpg"]);
    }

    #[test]
    fn sort_by_file_size() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("small.jpg"), b"a").unwrap();
        std::fs::write(dir.path().join("medium.jpg"), b"aaa").unwrap();
        std::fs::write(dir.path().join("large.jpg"), b"aaaaa").unwrap();
        let mut entries = scan_directory(dir.path());

        sort_entries(&mut entries, SortMode::FileSize, SortDirection::Ascending);

        let names: Vec<&str> = entries
            .iter()
            .map(|e| e.path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["small.jpg", "medium.jpg", "large.jpg"]);
    }

    #[test]
    fn sort_by_file_type_groups_extensions() {
        let dir = create_test_dir_with_files(&["b.png", "a.jpg", "c.gif", "d.jpg"]);
        let mut entries = scan_directory(dir.path());

        sort_entries(&mut entries, SortMode::FileType, SortDirection::Ascending);

        let exts: Vec<&str> = entries
            .iter()
            .map(|e| e.path.extension().unwrap().to_str().unwrap())
            .collect();
        // gif < jpg < png
        assert_eq!(exts, vec!["gif", "jpg", "jpg", "png"]);
    }
}
