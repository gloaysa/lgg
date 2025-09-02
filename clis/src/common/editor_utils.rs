use anyhow::Result;
use std::{fs, path::Path, process::Command};

pub fn resolve_editor(editor: &Option<String>) -> Result<String> {
    let editor = editor
        .as_deref()
        .map(str::to_string)
        .or_else(|| std::env::var("VISUAL").ok())
        .or_else(|| std::env::var("EDITOR").ok())
        .unwrap_or_else(|| "vim".into());
    Ok(editor)
}

pub fn create_editor_buffer(editor_cmd: &str) -> Result<String> {
    let file = tempfile::Builder::new()
        .prefix("lgg")
        .suffix(".md")
        .tempfile()?;

    let path = file.path().to_path_buf();
    open_file_in_editor(editor_cmd, &path)?;
    Ok(fs::read_to_string(&path)?)
}

pub fn open_file_in_editor(editor_cmd: &str, path: &Path) -> Result<()> {
    let status = Command::new(editor_cmd).arg(path).status()?;
    if !status.success() {
        anyhow::bail!("Editor exited with status {}", status);
    }
    Ok(())
}
