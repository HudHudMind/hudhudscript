//! Local `.hudpackages/` installation and cross-package import resolution.

use std::path::{Path, PathBuf};

use crate::{ResolvedDependency, Result, PACKAGES_DIR};

use super::Installer;

impl Installer {
    /// Install resolved dependencies into the project-local `.hudpackages/`
    /// directory so that cross-package imports can resolve at runtime.
    pub fn install_to_local(project_dir: &Path, deps: &[ResolvedDependency]) -> Result<()> {
        let packages_dir = project_dir.join(PACKAGES_DIR);
        std::fs::create_dir_all(&packages_dir)?;

        for dep in deps {
            let dest = packages_dir.join(&dep.name).join("lib");
            if !dest.exists() {
                std::fs::create_dir_all(&dest)?;
            }

            if let Some(ref source_path) = dep.resolved_path {
                if source_path.is_dir() {
                    super::utils::copy_hud_files(source_path, &dest)?;
                } else if source_path.is_file() {
                    let ext = source_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    if matches!(ext, "gz" | "tgz" | "tar")
                        || source_path
                            .to_str()
                            .map(|s| s.ends_with(".tar.gz") || s.ends_with(".hudpkg"))
                            .unwrap_or(false)
                    {
                        super::utils::extract_archive(source_path, &dest)?;
                    } else if ext == "hud" || ext == "hudhud" {
                        let target = dest.join(source_path.file_name().unwrap_or_default());
                        std::fs::copy(source_path, &target)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Resolve an import path of the form `"package-name"` to a directory
    /// inside `.hudpackages/<package>/lib/`.
    pub fn resolve_import(project_dir: &Path, package_name: &str) -> Option<PathBuf> {
        let lib_dir = project_dir
            .join(PACKAGES_DIR)
            .join(package_name)
            .join("lib");
        if lib_dir.is_dir() {
            Some(lib_dir)
        } else {
            None
        }
    }
}
