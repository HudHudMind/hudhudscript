//! Native library loader — manages search paths and caches loaded libraries.

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use crate::error::{NativeError, Result};
use crate::ffi::{NativeFunction, NativeLibrary};
use crate::types::NativeValue;

/// Manages discovery, loading, and caching of native shared libraries.
pub struct NativeLoader {
    /// Directories to search when resolving a library by name.
    search_paths: Vec<PathBuf>,
    /// Already-loaded libraries, keyed by logical name.
    loaded: HashMap<String, NativeLibrary>,
}

impl std::fmt::Debug for NativeLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeLoader")
            .field("search_paths", &self.search_paths)
            .field("loaded_libs", &self.loaded.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl Default for NativeLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeLoader {
    /// Create a new loader with the default search paths:
    ///
    /// 1. `./.hudpackages/native/lib/`
    /// 2. `/usr/local/lib/`
    /// 3. Every directory listed in `LD_LIBRARY_PATH`
    pub fn new() -> Self {
        let mut search_paths = vec![
            PathBuf::from("./.hudpackages/native/lib"),
            PathBuf::from("/usr/local/lib"),
        ];

        // Append LD_LIBRARY_PATH entries.
        if let Ok(ld_path) = env::var("LD_LIBRARY_PATH") {
            for dir in ld_path.split(':') {
                if !dir.is_empty() {
                    search_paths.push(PathBuf::from(dir));
                }
            }
        }

        Self {
            search_paths,
            loaded: HashMap::new(),
        }
    }

    /// Append an additional search directory.
    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }

    /// Return the current search paths (useful for diagnostics).
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }

    /// Check whether a library with the given name is already loaded.
    pub fn is_loaded(&self, name: &str) -> bool {
        self.loaded.contains_key(name)
    }

    /// Load a shared library by logical name.
    ///
    /// The loader searches each path in [`search_paths`](Self::search_paths) for a file
    /// named `lib{name}.so` (Linux), `lib{name}.dylib` (macOS), or `{name}.dll` (Windows).
    ///
    /// If the library has already been loaded, the cached handle is returned.
    pub fn load_library(&mut self, name: &str) -> Result<&NativeLibrary> {
        if self.loaded.contains_key(name) {
            return Ok(&self.loaded[name]);
        }

        let filename = platform_lib_filename(name);

        for dir in &self.search_paths {
            let candidate = dir.join(&filename);
            if candidate.is_file() {
                let lib = NativeLibrary::load(&candidate)?;
                self.loaded.insert(name.to_owned(), lib);
                return Ok(&self.loaded[name]);
            }
        }

        Err(NativeError::LibraryNotFound {
            name: name.to_owned(),
            search_paths: self
                .search_paths
                .iter()
                .map(|p| p.display().to_string())
                .collect(),
        })
    }

    /// Load a library from an explicit filesystem path (not searched).
    pub fn load_library_from_path(&mut self, name: &str, path: &Path) -> Result<&NativeLibrary> {
        if self.loaded.contains_key(name) {
            return Ok(&self.loaded[name]);
        }
        let lib = NativeLibrary::load(path)?;
        self.loaded.insert(name.to_owned(), lib);
        Ok(&self.loaded[name])
    }

    /// Get a mutable reference to a loaded library (e.g. to register functions).
    pub fn get_library_mut(&mut self, name: &str) -> Option<&mut NativeLibrary> {
        self.loaded.get_mut(name)
    }

    /// Get an immutable reference to a loaded library.
    pub fn get_library(&self, name: &str) -> Option<&NativeLibrary> {
        self.loaded.get(name)
    }

    /// Register a function on an already-loaded library.
    pub fn register_function(&mut self, lib_name: &str, func: NativeFunction) -> Result<()> {
        let lib = self
            .loaded
            .get_mut(lib_name)
            .ok_or_else(|| NativeError::LibraryNotLoaded {
                name: lib_name.to_owned(),
            })?;
        lib.register_function(func);
        Ok(())
    }

    /// Convenience: load library (if needed), then call a function on it.
    ///
    /// The function must already be registered on the library.
    pub fn call_function(
        &self,
        lib_name: &str,
        func_name: &str,
        args: &[NativeValue],
    ) -> Result<NativeValue> {
        let lib = self
            .loaded
            .get(lib_name)
            .ok_or_else(|| NativeError::LibraryNotLoaded {
                name: lib_name.to_owned(),
            })?;
        lib.call(func_name, args)
    }
}

/// Build the platform-specific shared library filename for a given base name.
fn platform_lib_filename(name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{name}.dylib")
    } else {
        format!("lib{name}.so")
    }
}
