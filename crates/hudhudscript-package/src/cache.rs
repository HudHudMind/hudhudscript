use crate::Result;
use flate2::read::GzDecoder;
use std::io::Cursor;
use std::path::PathBuf;
use tar::Archive;

#[derive(Clone)]
pub struct PackageCache {
    cache_dir: PathBuf,
}

impl PackageCache {
    pub fn new(cache_dir: &PathBuf) -> Result<Self> {
        std::fs::create_dir_all(cache_dir)?;
        Ok(Self {
            cache_dir: cache_dir.clone(),
        })
    }

    pub fn is_installed(&self, name: &str, version: &str) -> Result<bool> {
        let path = self.cache_dir.join(name).join(version);
        Ok(path.exists())
    }

    /// Extract a gzipped tar archive into the cache directory.
    pub fn install(&self, name: &str, version: &str, tarball: &[u8]) -> Result<()> {
        let path = self.cache_dir.join(name).join(version);
        std::fs::create_dir_all(&path)?;

        if !tarball.is_empty() {
            let cursor = Cursor::new(tarball);
            let decoder = GzDecoder::new(cursor);
            let mut archive = Archive::new(decoder);
            archive.unpack(&path)?;
        }

        Ok(())
    }

    pub fn remove(&self, name: &str) -> Result<()> {
        let path = self.cache_dir.join(name);
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }
        Ok(())
    }

    pub fn clean(&self) -> Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    pub fn list_installed(&self) -> Result<Vec<String>> {
        let mut packages = vec![];
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                packages.push(entry.file_name().to_string_lossy().to_string());
            }
        }
        Ok(packages)
    }

    /// Return the on-disk path where a given package version is extracted.
    pub fn package_path(&self, name: &str, version: &str) -> PathBuf {
        self.cache_dir.join(name).join(version)
    }

    /// Return the root cache directory.
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }
}
