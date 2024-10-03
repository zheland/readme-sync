use std::path::{Path, PathBuf};

use crate::{Manifest, TomlReadError};

/// A struct contains package manifest and its root path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Package {
    /// Package manifest.
    manifest: Manifest,
    /// Package root path.
    path: PathBuf,
}

impl Package {
    /// Creates a package from the specified path.
    pub fn from_path(path: PathBuf) -> Result<Self, TomlReadError> {
        Ok(Self {
            manifest: Manifest::from_package_path(&path)?,
            path,
        })
    }

    /// Creates a package from the manifest and package path.
    pub fn from_manifest_and_path(manifest: Manifest, path: PathBuf) -> Self {
        Self { manifest, path }
    }

    /// Returns a package manifest.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Returns a package path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns package relative readme path.
    pub fn relative_readme_path(&self) -> Option<&Path> {
        self.manifest.relative_readme_path(&self.path)
    }

    /// Returns package relative default readme path.
    pub fn default_relative_readme_path(&self) -> Option<&Path> {
        Manifest::default_readme_filename(&self.path)
    }
}
