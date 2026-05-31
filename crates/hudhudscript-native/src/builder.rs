#![allow(clippy::collapsible_match)]
//! Build native C/C++ projects via CMake or Conan.
//!
//! [`NativeBuilder`] shells out to `cmake` and `conan` to compile native
//! dependencies and returns the path to the resulting shared library.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::BuildType;
use crate::error::{NativeError, Result};

/// Builder that compiles C/C++ projects and produces shared libraries.
#[derive(Debug, Clone)]
pub struct NativeBuilder {
    /// Root directory of the HudHudScript project.
    pub project_dir: PathBuf,
    /// Directory where build artifacts are placed.
    pub build_dir: PathBuf,
    /// Build configuration (Debug / Release / RelWithDebInfo).
    pub build_type: BuildType,
}

impl NativeBuilder {
    /// Create a new builder with the given project root and build type.
    ///
    /// The build directory defaults to `{project_dir}/.hudpackages/native/build`.
    pub fn new(project_dir: PathBuf, build_type: BuildType) -> Self {
        let build_dir = project_dir.join(".hudpackages/native/build");
        Self {
            project_dir,
            build_dir,
            build_type,
        }
    }

    /// Build a CMake project and return the path to the produced shared library.
    ///
    /// Runs:
    /// ```text
    /// cmake -S <cmake_dir> -B <build_dir> -DCMAKE_BUILD_TYPE=<type> -DBUILD_SHARED_LIBS=ON
    /// cmake --build <build_dir> --config <type>
    /// ```
    ///
    /// The resulting `.so`/`.dylib`/`.dll` is expected in `<build_dir>/lib/` or `<build_dir>/`.
    pub fn build_cmake(&self, cmake_dir: &Path) -> Result<PathBuf> {
        let build_subdir = self.build_dir.join("cmake");
        std::fs::create_dir_all(&build_subdir).map_err(|e| NativeError::BuildError {
            message: format!("failed to create build directory: {e}"),
        })?;

        let bt = self.build_type.as_cmake_str();

        // Configure
        let status = Command::new("cmake")
            .arg("-S")
            .arg(cmake_dir)
            .arg("-B")
            .arg(&build_subdir)
            .arg(format!("-DCMAKE_BUILD_TYPE={bt}"))
            .arg("-DBUILD_SHARED_LIBS=ON")
            .status()
            .map_err(|e| NativeError::BuildError {
                message: format!("failed to run cmake configure: {e}"),
            })?;

        if !status.success() {
            return Err(NativeError::BuildError {
                message: format!("cmake configure failed with exit code {status}"),
            });
        }

        // Build
        let status = Command::new("cmake")
            .arg("--build")
            .arg(&build_subdir)
            .arg("--config")
            .arg(bt)
            .status()
            .map_err(|e| NativeError::BuildError {
                message: format!("failed to run cmake build: {e}"),
            })?;

        if !status.success() {
            return Err(NativeError::BuildError {
                message: format!("cmake build failed with exit code {status}"),
            });
        }

        // Try to locate the shared library in common output directories.
        for candidate_dir in &[build_subdir.join("lib"), build_subdir.clone()] {
            if let Some(lib) = find_shared_lib_in(candidate_dir) {
                // Copy to the canonical native lib output dir.
                let output_dir = self.output_lib_dir();
                std::fs::create_dir_all(&output_dir).ok();
                let dest = output_dir.join(lib.file_name().unwrap());
                std::fs::copy(&lib, &dest).map_err(|e| NativeError::BuildError {
                    message: format!("failed to copy built library: {e}"),
                })?;
                return Ok(dest);
            }
        }

        Err(NativeError::BuildError {
            message: "cmake build completed but no shared library found in output".into(),
        })
    }

    /// Install a Conan package and return the path to the produced shared library.
    ///
    /// Runs:
    /// ```text
    /// conan install <conan_ref> --output-folder=<build_dir>/conan -s build_type=<type> --build=missing
    /// ```
    pub fn build_conan(&self, conan_ref: &str) -> Result<PathBuf> {
        let conan_dir = self.build_dir.join("conan");
        std::fs::create_dir_all(&conan_dir).map_err(|e| NativeError::BuildError {
            message: format!("failed to create conan output directory: {e}"),
        })?;

        let bt = self.build_type.as_cmake_str();

        let status = Command::new("conan")
            .arg("install")
            .arg(conan_ref)
            .arg(format!("--output-folder={}", conan_dir.display()))
            .arg(format!("-s:h=build_type={bt}"))
            .arg("--build=missing")
            .status()
            .map_err(|e| NativeError::BuildError {
                message: format!("failed to run conan install: {e}"),
            })?;

        if !status.success() {
            return Err(NativeError::BuildError {
                message: format!("conan install failed with exit code {status}"),
            });
        }

        // Look for shared libs in the conan output tree.
        let lib_dir = conan_dir.join("lib");
        if let Some(lib) = find_shared_lib_in(&lib_dir) {
            let output_dir = self.output_lib_dir();
            std::fs::create_dir_all(&output_dir).ok();
            let dest = output_dir.join(lib.file_name().unwrap());
            std::fs::copy(&lib, &dest).map_err(|e| NativeError::BuildError {
                message: format!("failed to copy conan library: {e}"),
            })?;
            return Ok(dest);
        }

        Err(NativeError::BuildError {
            message: "conan install completed but no shared library found".into(),
        })
    }

    /// The canonical output directory for native libraries.
    pub fn output_lib_dir(&self) -> PathBuf {
        self.project_dir.join(".hudpackages/native/lib")
    }
}

/// Scan a directory for the first shared library file (`.so`, `.dylib`, `.dll`).
fn find_shared_lib_in(dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext {
                "so" | "dylib" | "dll" => return Some(path),
                _ => {}
            }
        }
        // Also match versioned .so files like libfoo.so.1.2.3
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.contains(".so.") {
                return Some(path);
            }
        }
    }
    None
}
