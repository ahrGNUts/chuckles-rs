use std::path::{Path, PathBuf};

use crate::config::{SortDirection, SortMode};
use crate::viewer::directory::{self, DirEntry};

/// Manages the sorted list of images in a directory and current position.
pub struct ImageList {
    entries: Vec<DirEntry>,
    current_index: Option<usize>,
    sort_mode: SortMode,
    sort_direction: SortDirection,
}

impl ImageList {
    pub fn new(sort_mode: SortMode, sort_direction: SortDirection) -> Self {
        Self {
            entries: Vec::new(),
            current_index: None,
            sort_mode,
            sort_direction,
        }
    }

    /// Populate the list by scanning a directory and position on the given file.
    pub fn load_directory(&mut self, dir: &Path, current_file: &Path) {
        self.entries = directory::scan_directory(dir);
        directory::sort_entries(&mut self.entries, self.sort_mode, self.sort_direction);
        self.current_index = self.entries.iter().position(|e| e.path == current_file);
    }

    /// Change sort mode and re-sort, keeping the current file selected.
    pub fn set_sort(&mut self, mode: SortMode, direction: SortDirection) {
        let current_path = self.current_path().map(|p| p.to_path_buf());
        self.sort_mode = mode;
        self.sort_direction = direction;
        directory::sort_entries(&mut self.entries, mode, direction);
        if let Some(path) = current_path {
            self.current_index = self.entries.iter().position(|e| e.path == path);
        }
    }

    /// Toggle sort direction between ascending and descending.
    pub fn toggle_direction(&mut self) {
        let new_dir = match self.sort_direction {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        };
        self.set_sort(self.sort_mode, new_dir);
    }

    pub fn current_path(&self) -> Option<&Path> {
        self.current_index
            .and_then(|i| self.entries.get(i))
            .map(|e| e.path.as_path())
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Move to next image. Returns true if position changed.
    pub fn next(&mut self) -> bool {
        match self.current_index {
            Some(i) if i + 1 < self.entries.len() => {
                self.current_index = Some(i + 1);
                true
            }
            _ => false,
        }
    }

    /// Move to previous image. Returns true if position changed.
    pub fn prev(&mut self) -> bool {
        match self.current_index {
            Some(i) if i > 0 => {
                self.current_index = Some(i - 1);
                true
            }
            _ => false,
        }
    }

    /// Jump to the first image. Returns true if position changed.
    pub fn first(&mut self) -> bool {
        if self.entries.is_empty() {
            return false;
        }
        let changed = self.current_index != Some(0);
        self.current_index = Some(0);
        changed
    }

    /// Jump to the last image. Returns true if position changed.
    pub fn last(&mut self) -> bool {
        if self.entries.is_empty() {
            return false;
        }
        let last = self.entries.len() - 1;
        let changed = self.current_index != Some(last);
        self.current_index = Some(last);
        changed
    }

    /// Get the path at a specific index.
    pub fn path_at(&self, index: usize) -> Option<&Path> {
        self.entries.get(index).map(|e| e.path.as_path())
    }

    /// Navigate to a specific index.
    pub fn go_to(&mut self, index: usize) -> bool {
        if index < self.entries.len() {
            let changed = self.current_index != Some(index);
            self.current_index = Some(index);
            changed
        } else {
            false
        }
    }

    /// Get paths of adjacent images for preloading (prev, next).
    pub fn adjacent_paths(&self) -> (Option<PathBuf>, Option<PathBuf>) {
        let prev = self
            .current_index
            .and_then(|i| i.checked_sub(1))
            .and_then(|i| self.entries.get(i))
            .map(|e| e.path.clone());
        let next = self
            .current_index
            .and_then(|i| i.checked_add(1))
            .and_then(|i| self.entries.get(i))
            .map(|e| e.path.clone());
        (prev, next)
    }

    pub fn sort_mode(&self) -> SortMode {
        self.sort_mode
    }

    pub fn sort_direction(&self) -> SortDirection {
        self.sort_direction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_populated_list(filenames: &[&str]) -> (tempfile::TempDir, ImageList) {
        let dir = tempfile::tempdir().unwrap();
        for name in filenames {
            std::fs::write(dir.path().join(name), b"fake").unwrap();
        }
        let mut list = ImageList::new(SortMode::Name, SortDirection::Ascending);
        let first_file = dir.path().join(filenames[0]);
        list.load_directory(dir.path(), &first_file);
        (dir, list)
    }

    #[test]
    fn load_directory_positions_on_current_file() {
        let dir = tempfile::tempdir().unwrap();
        for name in &["a.jpg", "b.jpg", "c.jpg"] {
            std::fs::write(dir.path().join(name), b"fake").unwrap();
        }
        let mut list = ImageList::new(SortMode::Name, SortDirection::Ascending);

        list.load_directory(dir.path(), &dir.path().join("b.jpg"));

        assert_eq!(list.current_index(), Some(1));
        assert_eq!(
            list.current_path()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            "b.jpg"
        );
    }

    #[test]
    fn next_advances_position() {
        let (_dir, mut list) = create_populated_list(&["a.jpg", "b.jpg", "c.jpg"]);

        let changed = list.next();

        assert!(changed);
        assert_eq!(
            list.current_path()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            "b.jpg"
        );
    }

    #[test]
    fn next_at_end_does_not_advance() {
        let (_dir, mut list) = create_populated_list(&["a.jpg", "b.jpg"]);
        list.last();

        let changed = list.next();

        assert!(!changed);
        assert_eq!(
            list.current_path()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            "b.jpg"
        );
    }

    #[test]
    fn prev_moves_backward() {
        let (_dir, mut list) = create_populated_list(&["a.jpg", "b.jpg", "c.jpg"]);
        list.last();

        let changed = list.prev();

        assert!(changed);
        assert_eq!(
            list.current_path()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            "b.jpg"
        );
    }

    #[test]
    fn prev_at_start_does_not_move() {
        let (_dir, mut list) = create_populated_list(&["a.jpg", "b.jpg"]);

        let changed = list.prev();

        assert!(!changed);
        assert_eq!(
            list.current_path()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            "a.jpg"
        );
    }

    #[test]
    fn first_jumps_to_beginning() {
        let (_dir, mut list) = create_populated_list(&["a.jpg", "b.jpg", "c.jpg"]);
        list.last();

        let changed = list.first();

        assert!(changed);
        assert_eq!(list.current_index(), Some(0));
    }

    #[test]
    fn last_jumps_to_end() {
        let (_dir, mut list) = create_populated_list(&["a.jpg", "b.jpg", "c.jpg"]);

        let changed = list.last();

        assert!(changed);
        assert_eq!(list.current_index(), Some(2));
    }

    #[test]
    fn go_to_navigates_to_index() {
        let (_dir, mut list) = create_populated_list(&["a.jpg", "b.jpg", "c.jpg"]);

        let changed = list.go_to(2);

        assert!(changed);
        assert_eq!(
            list.current_path()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            "c.jpg"
        );
    }

    #[test]
    fn go_to_out_of_bounds_returns_false() {
        let (_dir, mut list) = create_populated_list(&["a.jpg"]);

        let changed = list.go_to(5);

        assert!(!changed);
    }

    #[test]
    fn len_returns_entry_count() {
        let (_dir, list) = create_populated_list(&["a.jpg", "b.jpg"]);

        assert_eq!(list.len(), 2);
    }

    #[test]
    fn toggle_direction_reverses_sort() {
        let (_dir, mut list) = create_populated_list(&["a.jpg", "b.jpg", "c.jpg"]);
        assert_eq!(
            list.current_path()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            "a.jpg"
        );

        list.toggle_direction();

        // After reversing, a.jpg should still be selected but at a different index
        assert_eq!(
            list.current_path()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            "a.jpg"
        );
        assert_eq!(list.sort_direction(), SortDirection::Descending);
    }

    #[test]
    fn set_sort_preserves_current_selection() {
        let (_dir, mut list) = create_populated_list(&["a.jpg", "b.jpg", "c.jpg"]);
        list.go_to(1); // b.jpg

        list.set_sort(SortMode::Name, SortDirection::Descending);

        assert_eq!(
            list.current_path()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            "b.jpg"
        );
    }

    #[test]
    fn adjacent_paths_returns_neighbors() {
        let (_dir, mut list) = create_populated_list(&["a.jpg", "b.jpg", "c.jpg"]);
        list.go_to(1); // b.jpg

        let (prev, next) = list.adjacent_paths();

        assert!(prev.unwrap().ends_with("a.jpg"));
        assert!(next.unwrap().ends_with("c.jpg"));
    }

    #[test]
    fn adjacent_paths_at_start_has_no_prev() {
        let (_dir, list) = create_populated_list(&["a.jpg", "b.jpg"]);

        let (prev, next) = list.adjacent_paths();

        assert!(prev.is_none());
        assert!(next.unwrap().ends_with("b.jpg"));
    }

    #[test]
    fn adjacent_paths_at_end_has_no_next() {
        let (_dir, mut list) = create_populated_list(&["a.jpg", "b.jpg"]);
        list.last();

        let (prev, next) = list.adjacent_paths();

        assert!(prev.unwrap().ends_with("a.jpg"));
        assert!(next.is_none());
    }

    #[test]
    fn empty_list_navigation_is_safe() {
        let mut list = ImageList::new(SortMode::Name, SortDirection::Ascending);

        assert!(!list.next());
        assert!(!list.prev());
        assert!(!list.first());
        assert!(!list.last());
        assert!(list.current_path().is_none());
        assert!(list.is_empty());
    }

    #[test]
    fn natural_sort_orders_numbered_files_correctly() {
        let dir = tempfile::tempdir().unwrap();
        for name in &["img10.jpg", "img2.jpg", "img1.jpg", "img20.jpg"] {
            std::fs::write(dir.path().join(name), b"fake").unwrap();
        }
        let mut list = ImageList::new(SortMode::Name, SortDirection::Ascending);
        list.load_directory(dir.path(), &dir.path().join("img1.jpg"));

        let names: Vec<String> = (0..list.len())
            .map(|i| {
                list.path_at(i)
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect();

        assert_eq!(
            names,
            vec!["img1.jpg", "img2.jpg", "img10.jpg", "img20.jpg"]
        );
    }
}
