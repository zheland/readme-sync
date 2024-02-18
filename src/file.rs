#[cfg(feature = "thiserror")]
use std::io;
use std::path::{Path, PathBuf};
use std::string::String;

#[cfg(feature = "thiserror")]
use thiserror::Error;

/// File path and its contents.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct File {
    path: PathBuf,
    text: String,
}

impl File {
    /// Reads file from the specified path.
    #[cfg(feature = "thiserror")]
    pub fn from_path(path: PathBuf, root: Option<&Path>) -> Result<Self, FileFromPathError> {
        use std::fs;
        let content = match root {
            Some(root) => fs::read_to_string(root.join(&path)),
            None => fs::read_to_string(&path),
        };
        match content {
            Ok(text) => Ok(Self { path, text }),
            Err(err) => Err(FileFromPathError::IoError { err, path }),
        }
    }

    /// Creates file from the specified path and text.
    pub fn from_path_and_text(path: PathBuf, text: String) -> Self {
        Self { path, text }
    }

    /// Returns file text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns file path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// An error which can occur when reading a file from the specified path.
#[cfg(feature = "thiserror")]
#[derive(Debug, Error)]
pub enum FileFromPathError {
    /// File reading failed.
    #[error("Failed to read file at `{path}`: {err}")]
    IoError {
        /// File path.
        path: PathBuf,
        /// Rust `io::Error`.
        #[source]
        err: io::Error,
    },
}
