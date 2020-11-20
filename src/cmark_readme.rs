#![cfg(feature = "pulldown-cmark")]

#[cfg(feature = "thiserror")]
use std::path::Path;
use std::sync::Arc;

#[cfg(feature = "thiserror")]
use thiserror::Error;

use crate::{CMarkData, CMarkDataIter, File, Manifest, Package};
#[cfg(feature = "thiserror")]
use crate::{DisallowUrlsWithPrefixError, FileFromPathError};

/// Parsed readme Markdown with optionally specified package path and package manifest.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CMarkReadme<P, M> {
    data: CMarkData,
    package_path: P,
    manifest: M,
}

#[cfg(feature = "thiserror")]
impl<'a> CMarkReadme<&'a Path, &'a Manifest> {
    /// Creates readme from package.
    ///
    /// It reads readme file by path specified in the package manifest.
    pub fn from_package(package: &'a Package) -> Result<Self, CMarkReadmeFromPackageError> {
        let path = package
            .relative_readme_path()
            .ok_or(CMarkReadmeFromPackageError::NotFound)?;
        let file = Arc::new(File::from_path(path.to_path_buf(), Some(package.path()))?);
        let package_path = package.path();
        let manifest = package.manifest();
        Ok(Self::from_file_and_package_path_and_manifest(
            file,
            package_path,
            manifest,
        ))
    }
}

impl<'a> CMarkReadme<(), ()> {
    /// Creates readme from file.
    pub fn from_file(file: Arc<File>) -> Self {
        Self::from_file_and_package_path_and_manifest(file, (), ())
    }
}

impl<'a, P, M> CMarkReadme<P, M> {
    /// Adding the specified package path to the readme.
    pub fn with_package_path(self, package_path: &'a Package) -> CMarkReadme<&'a Package, M> {
        CMarkReadme {
            data: self.data,
            package_path,
            manifest: self.manifest,
        }
    }

    /// Adding the specified manifest to the readme.
    pub fn with_manifest(self, manifest: &'a Manifest) -> CMarkReadme<P, &'a Manifest> {
        CMarkReadme {
            data: self.data,
            package_path: self.package_path,
            manifest,
        }
    }

    /// Creates readme from file, package path and manifest.
    pub fn from_file_and_package_path_and_manifest(
        file: Arc<File>,
        package_path: P,
        manifest: M,
    ) -> Self {
        let data = CMarkData::from_file(file);
        Self::from_data_and_package_path_and_manifest(data, package_path, manifest)
    }

    /// Creates readme from CMark items, package path and manifest.
    pub fn from_data_and_package_path_and_manifest(
        data: CMarkData,
        package_path: P,
        manifest: M,
    ) -> Self {
        Self {
            data,
            package_path,
            manifest,
        }
    }

    /// Returns CMark items.
    pub fn data(&self) -> &CMarkData {
        &self.data
    }

    /// Returns the package path.
    pub fn package_path(&self) -> &P {
        &self.package_path
    }

    /// Returns the manifest.
    pub fn manifest(&self) -> &M {
        &self.manifest
    }

    /// Iterate over `CMarkItem`s.
    pub fn iter(&self) -> CMarkDataIter<'_> {
        self.data.iter()
    }

    fn map<F>(mut self, func: F) -> CMarkReadme<P, M>
    where
        F: FnOnce(CMarkData) -> CMarkData,
    {
        self.data = func(self.data);
        self
    }

    #[cfg(feature = "thiserror")]
    fn map_result<F, E>(mut self, func: F) -> Result<CMarkReadme<P, M>, E>
    where
        F: FnOnce(CMarkData) -> Result<CMarkData, E>,
    {
        self.data = func(self.data)?;
        Ok(self)
    }

    /// Concatenate adjacent text events.
    ///
    /// After readme and docs parsing some text events remain ununited.
    /// For example Rust attribute parser generate seperate text events
    /// for every line of source code, and pulldown_cmark generate
    /// seperate text events for character entity reference.
    pub fn concat_texts(self) -> CMarkReadme<P, M> {
        self.map(|data| data.concat_texts())
    }

    /// Removes first paragraph that contains only images and image-links,
    /// if the specified predicate returns true when passing image urls to it.
    pub fn remove_images_only_paragraph<F>(self, predicate: F) -> CMarkReadme<P, M>
    where
        F: FnMut(&[&str]) -> bool,
    {
        self.map(|data| data.remove_images_only_paragraph(predicate))
    }

    /// Removes first paragraph that contains only badges.
    #[cfg(feature = "glob")]
    pub fn remove_badges_paragraph(self) -> CMarkReadme<P, M> {
        self.map(|data| data.remove_badges_paragraph())
    }

    /// Remove section with the specified heading text and level and its subsections.
    pub fn remove_section(self, heading: &str, level: u32) -> Self {
        self.map(|data| data.remove_section(heading, level))
    }

    /// Remove sections with heading `Documentation` and level 2.
    pub fn remove_documentation_section(self) -> Self {
        self.map(|data| data.remove_documentation_section())
    }

    /// Returns self if absolute blob links to the specified repository not found,
    /// otherwise returns an error.
    #[cfg(feature = "thiserror")]
    pub fn disallow_absolute_blob_links(
        self,
        repository_url: &str,
    ) -> Result<CMarkReadme<P, M>, DisallowUrlsWithPrefixError> {
        self.map_result(|data| data.disallow_absolute_blob_links(repository_url))
    }

    /// Convert all relative links into absolute ones using
    /// the repository url as the root address.
    pub fn use_absolute_blob_urls(self, repository_url: &str) -> CMarkReadme<P, M> {
        self.map(|data| data.use_absolute_blob_urls(repository_url))
    }
}

#[cfg(feature = "thiserror")]
impl<'a, P> CMarkReadme<P, &'a Manifest> {
    /// Returns self if absolute blob links to the manifest repository not found,
    /// otherwise returns an error.
    pub fn disallow_absolute_repository_blob_links(
        self,
    ) -> Result<CMarkReadme<P, &'a Manifest>, DisallowAbsoluteRepositoryBlobLinksError> {
        let repository = self
            .manifest
            .package
            .repository
            .clone()
            .ok_or(DisallowAbsoluteRepositoryBlobLinksError::DocsUrlNotFound)?;
        Ok(self.disallow_absolute_blob_links(&repository)?)
    }

    /// Convert all relative links into absolute ones
    /// using the manifest repository url as the root address.
    pub fn use_absolute_repository_blob_urls(
        self,
    ) -> Result<CMarkReadme<P, &'a Manifest>, UseAbsoluteRepositoryBlobUrlsError> {
        let repository = self
            .manifest
            .package
            .repository
            .clone()
            .ok_or(UseAbsoluteRepositoryBlobUrlsError::DocsUrlNotFound)?;
        Ok(self.use_absolute_blob_urls(&repository))
    }
}

/// An error which can occur when creating readme from package.
#[cfg(feature = "thiserror")]
#[derive(Debug, Error)]
pub enum CMarkReadmeFromPackageError {
    /// File reading failed.
    #[error(transparent)]
    FileError(#[from] FileFromPathError),
    /// Readme file not found.
    #[error("CMarkReadme not found.")]
    NotFound,
}

/// An error which can occur when checking for disallowed repository blob links.
#[cfg(feature = "thiserror")]
#[derive(Clone, Debug, Error)]
pub enum DisallowAbsoluteRepositoryBlobLinksError {
    /// A disallowed prefix found.
    #[error(transparent)]
    DisallowUrlsWithPrefixError(#[from] DisallowUrlsWithPrefixError),
    /// Manifest does not contain `package.documentation` field.
    #[error("Manifest does not contain package.documentation field")]
    DocsUrlNotFound,
}

/// An error which can occur when converting relative links into absolute ones,
/// using the manifest repository url as the root address.
#[cfg(feature = "thiserror")]
#[derive(Clone, Copy, Debug, Error)]
pub enum UseAbsoluteRepositoryBlobUrlsError {
    #[error("Manifest does not contain package.documentation field")]
    DocsUrlNotFound,
}
