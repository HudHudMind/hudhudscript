//! .deb package builder
//!
//! Generates Debian package structure: DEBIAN/control, conffiles, postinst, prerm.

use crate::DeployError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a file to be included in the .deb package.
#[derive(Debug, Clone)]
pub struct DebFile {
    /// Source path on the build machine.
    pub source: PathBuf,
    /// Destination path inside the package (absolute, e.g. `/usr/bin/app`).
    pub dest: PathBuf,
}

/// Builder for a `.deb` package.
#[derive(Debug, Clone)]
pub struct DebPackage {
    pub name: String,
    pub version: String,
    pub description: String,
    pub maintainer: String,
    pub architecture: String,
    pub dependencies: Vec<String>,
    pub files: Vec<DebFile>,
    pub config_files: Vec<DebFile>,
    pub postinst: Option<String>,
    pub prerm: Option<String>,
    /// Extra control fields (e.g. Section, Priority, Homepage).
    pub extra_fields: HashMap<String, String>,
}

impl DebPackage {
    /// Create a new package with required metadata.
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        maintainer: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: description.into(),
            maintainer: maintainer.into(),
            architecture: "amd64".to_string(),
            dependencies: Vec::new(),
            files: Vec::new(),
            config_files: Vec::new(),
            postinst: None,
            prerm: None,
            extra_fields: HashMap::new(),
        }
    }

    /// Add a regular file to the package.
    pub fn add_file(&mut self, source: impl Into<PathBuf>, dest: impl Into<PathBuf>) {
        self.files.push(DebFile {
            source: source.into(),
            dest: dest.into(),
        });
    }

    /// Add a configuration file (listed in `conffiles`).
    pub fn add_config(&mut self, source: impl Into<PathBuf>, dest: impl Into<PathBuf>) {
        self.config_files.push(DebFile {
            source: source.into(),
            dest: dest.into(),
        });
    }

    /// Set the postinst maintainer script.
    pub fn set_postinst(&mut self, script: impl Into<String>) {
        self.postinst = Some(script.into());
    }

    /// Set the prerm maintainer script.
    pub fn set_prerm(&mut self, script: impl Into<String>) {
        self.prerm = Some(script.into());
    }

    /// Generate the `DEBIAN/control` file content.
    pub fn generate_control(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Package: {}\n", self.name));
        out.push_str(&format!("Version: {}\n", self.version));
        out.push_str(&format!("Architecture: {}\n", self.architecture));
        out.push_str(&format!("Maintainer: {}\n", self.maintainer));
        out.push_str(&format!("Description: {}\n", self.description));

        if !self.dependencies.is_empty() {
            out.push_str(&format!("Depends: {}\n", self.dependencies.join(", ")));
        }

        for (key, value) in &self.extra_fields {
            out.push_str(&format!("{}: {}\n", key, value));
        }

        out
    }

    /// Generate the `DEBIAN/conffiles` content (one absolute path per line).
    pub fn generate_conffiles(&self) -> String {
        self.config_files
            .iter()
            .map(|f| f.dest.display().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Build the .deb directory structure inside `output_dir`.
    ///
    /// Returns the path to the package root directory
    /// (`output_dir/<name>_<version>_<arch>`).
    ///
    /// The caller can then run `dpkg-deb --build` on the returned path.
    pub fn build(&self, output_dir: &Path) -> Result<PathBuf, DeployError> {
        let pkg_dir_name = format!("{}_{}_{}", self.name, self.version, self.architecture);
        let pkg_root = output_dir.join(&pkg_dir_name);
        let debian_dir = pkg_root.join("DEBIAN");

        // Create DEBIAN directory
        std::fs::create_dir_all(&debian_dir)
            .map_err(|e| DeployError::BuildFailed(format!("cannot create DEBIAN dir: {e}")))?;

        // Write control
        let control = self.generate_control();
        std::fs::write(debian_dir.join("control"), &control)
            .map_err(|e| DeployError::BuildFailed(format!("cannot write control: {e}")))?;

        // Write conffiles (only if there are config files)
        if !self.config_files.is_empty() {
            let conffiles = self.generate_conffiles();
            std::fs::write(debian_dir.join("conffiles"), &conffiles)
                .map_err(|e| DeployError::BuildFailed(format!("cannot write conffiles: {e}")))?;
        }

        // Write postinst
        if let Some(ref script) = self.postinst {
            std::fs::write(debian_dir.join("postinst"), script)
                .map_err(|e| DeployError::BuildFailed(format!("cannot write postinst: {e}")))?;
        }

        // Write prerm
        if let Some(ref script) = self.prerm {
            std::fs::write(debian_dir.join("prerm"), script)
                .map_err(|e| DeployError::BuildFailed(format!("cannot write prerm: {e}")))?;
        }

        // Copy regular files into the package tree
        for file in &self.files {
            let target = pkg_root.join(file.dest.strip_prefix("/").unwrap_or(&file.dest));
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    DeployError::BuildFailed(format!("cannot create dir for file: {e}"))
                })?;
            }
            std::fs::copy(&file.source, &target).map_err(|e| {
                DeployError::BuildFailed(format!(
                    "cannot copy {} -> {}: {e}",
                    file.source.display(),
                    target.display()
                ))
            })?;
        }

        // Copy config files into the package tree
        for file in &self.config_files {
            let target = pkg_root.join(file.dest.strip_prefix("/").unwrap_or(&file.dest));
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    DeployError::BuildFailed(format!("cannot create dir for config: {e}"))
                })?;
            }
            std::fs::copy(&file.source, &target).map_err(|e| {
                DeployError::BuildFailed(format!(
                    "cannot copy config {} -> {}: {e}",
                    file.source.display(),
                    target.display()
                ))
            })?;
        }

        Ok(pkg_root)
    }
}
