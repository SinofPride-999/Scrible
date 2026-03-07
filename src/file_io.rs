use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Open or create a file, returning its contents as a string.
pub fn open_or_create(path: &str) -> Result<(PathBuf, String)> {
    let p = PathBuf::from(path);
    let content = if p.exists() {
        fs::read_to_string(&p)
            .with_context(|| format!("Failed to read file: {}", path))?
    } else {
        // Create an empty file on disk
        fs::write(&p, "")
            .with_context(|| format!("Failed to create file: {}", path))?;
        String::new()
    };
    Ok((p, content))
}

/// Write `content` to `path`, overwriting the file.
pub fn save(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content)
        .with_context(|| format!("Failed to save file: {}", path.display()))
}
