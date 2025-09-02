use std::path::{Path, PathBuf};

/// Path to pending todos file based on the root dir
pub fn pending_todos_file(root: &Path) -> PathBuf {
    root.join(format!("pending_todos.md"))
}

/// Path to done todos file based on the root dir
pub fn done_todos_file(root: &Path) -> PathBuf {
    root.join(format!("done_todos.md"))
}
