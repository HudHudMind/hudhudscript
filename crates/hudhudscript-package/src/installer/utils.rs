//! Install options and filesystem helpers.

use std::path::{Path, PathBuf};

use crate::Result;

/// Install options
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    pub dev: bool,
    pub optional: bool,
    pub force: bool,
}

/// Extract a tar.gz package archive into the destination directory.
pub fn extract_archive(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(archive_path)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(dest)?;
    Ok(())
}

/// Recursively copy .hud/.hudhud files from source to destination.
pub fn copy_hud_files(source: &Path, dest: &Path) -> Result<()> {
    if let Ok(entries) = std::fs::read_dir(source) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();
            let target = dest.join(&file_name);

            if path.is_dir() {
                std::fs::create_dir_all(&target)?;
                copy_hud_files(&path, &target)?;
            } else if let Some(ext) = path.extension() {
                if ext == "hud" || ext == "hudhud" || ext == "toml" || ext == "json" {
                    std::fs::copy(&path, &target)?;
                }
            }
        }
    }
    Ok(())
}
