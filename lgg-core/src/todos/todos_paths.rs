use std::path::{Path, PathBuf};

/// Path to pending todos file based on the root dir
pub fn todos_file(root: &Path) -> PathBuf {
    root.join("todos.md".to_string())
}
