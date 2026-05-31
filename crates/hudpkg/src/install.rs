use std::fs;
use std::path::{Path, PathBuf};

use crate::{Resolved, INSTALL_DIR};

/// Install all resolved packages by copying their source to `.hudpkg/<name>/`.
pub(crate) fn install_resolved(resolved: &std::collections::BTreeMap<String, Resolved>) {
    let install_dir = PathBuf::from(INSTALL_DIR);
    if let Err(e) = fs::create_dir_all(&install_dir) {
        eprintln!("Error: could not create {}: {}", INSTALL_DIR, e);
        std::process::exit(1);
    }

    for (name, r) in resolved {
        let target = install_dir.join(name);
        print!("  Installing {} v{} ...", name, r.version);

        if target.exists() {
            if let Err(e) = fs::remove_dir_all(&target) {
                eprintln!(" FAILED (cleanup: {})", e);
                continue;
            }
        }

        match copy_dir_recursive(&r.source_path, &target) {
            Ok(count) => println!(" OK ({} files)", count),
            Err(e) => eprintln!(" FAILED ({})", e),
        }
    }
}

/// Recursively copy a directory. Returns the number of files copied.
pub(crate) fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<usize, String> {
    fs::create_dir_all(dst).map_err(|e| format!("mkdir {}: {}", dst.display(), e))?;

    let mut count = 0;
    let entries = fs::read_dir(src).map_err(|e| format!("read_dir {}: {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("read entry: {}", e))?;
        let file_type = entry.file_type().map_err(|e| format!("file_type: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            count += copy_dir_recursive(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            fs::copy(&src_path, &dst_path).map_err(|e| {
                format!(
                    "copy {} -> {}: {}",
                    src_path.display(),
                    dst_path.display(),
                    e
                )
            })?;
            count += 1;
        }
    }

    Ok(count)
}
