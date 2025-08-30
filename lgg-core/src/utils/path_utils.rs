use anyhow::Result;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

pub fn scan_dir_for_md_files(path: &Path) -> Result<Vec<PathBuf>> {
    let mut file_paths = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();

        if p.is_dir() {
            file_paths.extend(scan_dir_for_md_files(&p)?);
        } else if p.is_file() && is_markdown(&p) {
            file_paths.push(p);
        }
    }

    Ok(file_paths)
}

fn is_markdown(p: &Path) -> bool {
    p.extension()
        .and_then(OsStr::to_str)
        .map(|ext| ext.eq_ignore_ascii_case("md"))
        .unwrap_or(false)
}
