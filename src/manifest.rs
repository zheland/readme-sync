use std::collections::{HashMap, HashSet};
#[cfg(all(feature = "toml", feature = "thiserror"))]
use std::io;
use std::path::{Path, PathBuf};
use std::string::String;
use std::vec::Vec;

#[cfg(feature = "serde")]
use serde::Deserialize;
#[cfg(all(feature = "toml", feature = "thiserror"))]
use thiserror::Error;

/// Package manifest.
///
/// It includes only fields that are necessary for
/// locating and parsing readme and library documentation.
///
/// See <https://doc.rust-lang.org/cargo/reference/manifest.html> for more details.
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Manifest {
    /// Defines a package.
    pub package: ManifestPackage,
    /// Library target settings.
    pub lib: Option<ManifestLibTarget>,
    /// Binary target settings.
    pub bin: Option<Vec<ManifestBinTarget>>,
    /// Conditional compilation features.
    pub features: Option<HashMap<String, HashSet<String>>>,
    /// Package library dependencies.
    pub dependencies: Option<HashMap<String, ManifestDependency>>,
    /// Metadata that customize docs.rs builds.
    #[cfg_attr(feature = "serde", serde(rename = "package.metadata.docs.rs"))]
    pub docs_meta: Option<ManifestDocsRsMetadata>,
}

/// Package manifest `[package]` section.
///
/// See <https://doc.rust-lang.org/cargo/reference/manifest.html#the-package-section> for more details.
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ManifestPackage {
    /// The package name that is used to locate main binary,
    /// add package title, disallow package docs links, use absolute package docs links.
    pub name: String,
    /// The package version that is not used by current library but defined as a required by Cargo.
    pub version: String,
    /// The `documentation` field specifies a URL to a website hosting the crate's documentation.
    pub documentation: Option<String>,
    /// The `readme` field specifies a path to a readme file in the package root (relative to this Cargo.toml).
    pub readme: Option<ManifestReadmePath>,
    /// The `repository` field specifies a URL to the source repository for the package.
    pub repository: Option<String>,
}

/// Package manifest `[lib]` section.
///
/// See <https://doc.rust-lang.org/cargo/reference/cargo-targets.html#library> for more details.
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ManifestLibTarget {
    /// The name of the target.
    pub name: Option<String>,
    /// The source file of the target.
    pub path: Option<String>,
    /// Is documented by default.
    pub doc: Option<bool>,
}

/// Package manifest `[[bin]]` section.
///
/// See <https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries> for more details.
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ManifestBinTarget {
    /// The name of the target.
    pub name: String,
    /// The source file of the target.
    pub path: Option<String>,
    /// Is documented by default.
    pub doc: Option<bool>,
}

/// Package manifest dependency.
///
/// See <https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html> for more details.
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ManifestDependency {
    /// Is the dependency is optional and therefore adds a feature with the specified name.
    pub optional: Option<bool>,
}

/// Manifest metadata that customize docs.rs builds.
///
/// See <https://docs.rs/about/metadata> for more details
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestDocsRsMetadata {
    /// Features to pass to Cargo (default: []).alloc
    pub features: Option<HashSet<String>>,
    /// Whether to pass `--all-features` to Cargo (default: false).
    #[cfg_attr(feature = "serde", serde(rename = "all-features"))]
    pub all_features: Option<bool>,
    /// Whether to pass `--no-default-features` to Cargo (default: false).
    #[cfg_attr(feature = "serde", serde(rename = "no-default-features"))]
    pub no_default_features: Option<bool>,
    /// Target to test build on, used as the default landing page.
    #[cfg_attr(feature = "serde", serde(rename = "default-target"))]
    pub default_target: Option<String>,
    /// Targets to build.
    pub targets: Option<Vec<String>>,
}

/// The optional Manifest `readme` field that allows string or boolean value.
///
/// If `readme` field is not specified, and a file named README.md, README.txt or README
/// exists in the package root, then the name of that file will be used.
///
/// See <https://doc.rust-lang.org/cargo/reference/manifest.html#the-readme-field> for more details.
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManifestReadmePath {
    /// Readme path.
    Path(PathBuf),
    /// If the field is set to true, a default value of README.md will be assumed.
    /// If the field is set to false, a readme file is defined as absent.
    Bool(bool),
}

impl Manifest {
    /// Creates simple manifest from package name and version.
    pub fn from_name_and_version(name: String, version: String) -> Self {
        Manifest {
            package: ManifestPackage {
                name,
                version,
                repository: None,
                documentation: None,
                readme: None,
            },
            lib: None,
            bin: None,
            features: None,
            dependencies: None,
            docs_meta: None,
        }
    }

    /// Creates manifest from `Cargo.toml` file contents.
    #[cfg(all(feature = "toml", feature = "serde", feature = "thiserror"))]
    pub fn from_cargo_toml_content(content: &str) -> Result<Self, TomlParseError> {
        Ok(toml::from_str(&content)?)
    }

    /// Reads manifest from a specified file path.
    #[cfg(all(feature = "toml", feature = "serde", feature = "thiserror"))]
    pub fn from_cargo_toml_path(path: &Path) -> Result<Self, TomlReadError> {
        let content = std::fs::read_to_string(path).map_err(|err| TomlReadError::IoError {
            path: path.to_path_buf(),
            err,
        })?;
        Self::from_cargo_toml_content(&content).map_err(|err| TomlReadError::ParseError {
            path: path.to_path_buf(),
            err,
        })
    }

    /// Reads manifest from the `Cargo.toml` file in the specified package path.
    #[cfg(all(feature = "toml", feature = "serde", feature = "thiserror"))]
    pub fn from_package_path(path: &Path) -> Result<Self, TomlReadError> {
        Self::from_cargo_toml_path(&path.join("Cargo.toml"))
    }

    /// Returns package relative readme path.
    pub fn relative_readme_path(&self, root: &Path) -> Option<&Path> {
        match &self.package.readme {
            Some(value) => match value {
                ManifestReadmePath::Bool(false) => None,
                ManifestReadmePath::Bool(true) => Some(Path::new("README.md")),
                ManifestReadmePath::Path(value) => Some(value),
            },
            None => Manifest::default_readme_filename(root),
        }
    }

    /// Returns package relative default readme path.
    pub fn default_readme_filename(root: &Path) -> Option<&'static Path> {
        const DEFAULT_FILES: [&str; 3] = ["README.md", "README.txt", "README"];

        for &filename in DEFAULT_FILES.iter() {
            if root.join(filename).is_file() {
                return Some(Path::new(filename));
            }
        }

        None
    }

    /// Returns `true` if the package's library is documented by default.
    ///
    /// See <https://doc.rust-lang.org/cargo/commands/cargo-doc.html> for more details.
    pub fn is_lib_documented_by_default(&self) -> bool {
        self.lib.as_ref().and_then(|lib| lib.doc).unwrap_or(true)
    }

    /// Returns package relative library file path.
    ///
    /// See <https://doc.rust-lang.org/cargo/commands/cargo-doc.html> for more details.
    pub fn relative_lib_path(&self) -> &Path {
        Path::new(
            self.lib
                .as_ref()
                .and_then(|lib| lib.path.as_deref())
                .unwrap_or("src/lib.rs"),
        )
    }

    /// Returns package relative default binary file path.
    ///
    /// See <https://doc.rust-lang.org/cargo/commands/cargo-doc.html> for more details.
    pub fn default_relative_bin_path(&self) -> &'static Path {
        Path::new("src/main.rs")
    }

    /// Returns package relative binary file path by the specified binary target name.
    ///
    /// See <https://doc.rust-lang.org/cargo/commands/cargo-doc.html> for more details.
    #[cfg(all(feature = "toml", feature = "thiserror"))]
    pub fn relative_bin_path(&self, name: &str) -> Result<PathBuf, BinPathError> {
        use std::string::ToString;

        let mut bins = self.bin.iter().flatten().filter(|bin| bin.name == name);
        match (bins.next(), bins.next()) {
            (Some(_), Some(_)) => Err(BinPathError::SpecifiedMoreThanOnce(name.to_string())),
            (Some(bin), None) => Ok(bin.path.as_ref().map_or_else(
                || PathBuf::from("src/bin").join(Path::new(&bin.name)),
                PathBuf::from,
            )),
            (None, None) => {
                if name == self.package.name {
                    Ok(PathBuf::from("src/main.rs"))
                } else {
                    Err(BinPathError::NotFound(name.to_string()))
                }
            }
            (None, Some(_)) => unreachable!(),
        }
    }

    /// Returns package default library or binary target.
    ///
    /// See <https://doc.rust-lang.org/cargo/commands/cargo-doc.html> for more details.
    pub fn default_relative_target_path(&self) -> &Path {
        if self.is_lib_documented_by_default() {
            self.relative_lib_path()
        } else {
            self.default_relative_bin_path()
        }
    }

    /// Returns package target used for docs.rs builds.
    ///
    /// See <https://docs.rs/about/metadata> for more details.
    pub fn docs_rs_default_target(&self) -> &str {
        const DEFAULT_TARGET: &str = "x86_64-unknown-linux-gnu";

        if let Some(docs_meta) = &self.docs_meta {
            if let Some(default_target) = &docs_meta.default_target {
                return default_target;
            }
            if let Some(targets) = &docs_meta.targets {
                if let Some(first_target) = targets.first() {
                    return first_target;
                }
            }
        }
        DEFAULT_TARGET
    }

    /// Returns a default package features.
    pub fn default_features(&self) -> HashSet<&str> {
        use core::ops::Deref;

        if let Some(features) = self.features.as_ref() {
            if let Some(default_features) = features.get("default") {
                return default_features.iter().map(Deref::deref).collect();
            }
        }
        HashSet::new()
    }

    /// Returns all package features.
    pub fn all_features(&self) -> HashSet<&str> {
        use core::ops::Deref;

        let mut all_features = HashSet::new();
        if let Some(features) = self.features.as_ref() {
            all_features.extend(features.keys().map(Deref::deref));
        }
        if let Some(dependencies) = self.dependencies.as_ref() {
            all_features.extend(
                dependencies
                    .iter()
                    .filter_map(|(name, dep)| match dep.optional {
                        Some(true) => Some(name.deref()),
                        _ => None,
                    }),
            );
        }
        all_features
    }

    /// Returns package features used for docs.rs builds.
    ///
    /// See <https://docs.rs/about/metadata> for more details.
    pub fn docs_rs_features(&self) -> HashSet<&str> {
        use core::ops::Deref;

        let all_features = self
            .docs_meta
            .as_ref()
            .and_then(|docs_meta| docs_meta.all_features)
            .unwrap_or(false);
        if all_features {
            return self.all_features();
        }

        let no_default_features = self
            .docs_meta
            .as_ref()
            .and_then(|docs_meta| docs_meta.no_default_features)
            .unwrap_or(false);
        let features = self
            .docs_meta
            .as_ref()
            .and_then(|docs_meta| docs_meta.features.as_ref())
            .map(|features| features.iter().map(Deref::deref).collect());

        match (no_default_features, features) {
            (true, Some(features)) => features,
            (true, None) => HashSet::new(),
            (false, Some(features)) => features.union(&self.default_features()).copied().collect(),
            (false, None) => self.default_features(),
        }
    }
}

/// An error which can occur when parsing manifest from toml file.
#[cfg(all(feature = "toml", feature = "thiserror"))]
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum TomlParseError {
    /// Toml parse error
    #[error(transparent)]
    ParseError(#[from] toml::de::Error),
}

/// An error which can occur when reading manifest from the specified file path.
#[cfg(all(feature = "toml", feature = "thiserror"))]
#[derive(Debug, Error)]
pub enum TomlReadError {
    /// File reading failed.
    #[error("Failed to read toml at `{path}`: {err}")]
    IoError {
        /// File path.
        path: PathBuf,
        /// Rust `io::Error`.
        err: io::Error,
    },
    /// File parsing failed.
    #[error("Failed to parse toml at `{path}`: {err}")]
    ParseError {
        /// File path.
        path: PathBuf,
        /// The corresponding parse error.
        err: TomlParseError,
    },
}

/// An error which can occur when locating the binary file path by the specified target name.
#[cfg(all(feature = "toml", feature = "thiserror"))]
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum BinPathError {
    /// The binary specified by the target name is not found.
    #[error("Binary `{0}` not found.")]
    NotFound(String),
    /// The binary specified by the target name is specified more than once.
    #[error("Binary `{0}` specified more than once.")]
    SpecifiedMoreThanOnce(String),
}
